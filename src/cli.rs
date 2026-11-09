use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use crate::corpus::{threshold_basis_points, AddOutcome, CheckOptions, Database};
use crate::error::Error;
use crate::files::load_inputs;
use crate::method::{
    CORPUS_SCHEMA, DEFAULT_MIN_CONTAINMENT, DISCLAIMER, PERSISTED_TEXT_NOTE, PRODUCT,
};
use crate::report::{bundle, render_json, render_markdown, review_file, Bundle};

#[derive(Parser)]
#[command(
    name = "course-integrity",
    version,
    about = "Local-first coursework integrity precheck (descriptive signals only)",
    long_about = "Scan UTF-8 .txt and .md drafts and, if you pass --db, compare them with a local SQLite corpus you build yourself.\n\nReports describe sentence length, repetition, vocabulary, transition phrases, and exact or near-exact local overlap.\n\nThese are descriptive review signals. They are not an AI probability, not a calibrated risk score, and not a plagiarism verdict. The tool does not use the network.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Describe a draft and, with --db, compare it to a local corpus.
    Scan(ScanArgs),
    /// Build and query the local corpus. The database stores original text.
    #[command(subcommand)]
    Corpus(CorpusCommand),
}

#[derive(Subcommand)]
enum CorpusCommand {
    /// Store UTF-8 .txt or .md files in a local database.
    Add(AddArgs),
    /// Same report as `scan --db`.
    Check(ScanArgs),
    /// List stored documents without printing their text.
    List(DbArgs),
    /// Show how much text this local database covers.
    Stats(DbArgs),
}

#[derive(Parser)]
struct ScanArgs {
    /// File or directory of UTF-8 .txt and .md drafts.
    path: PathBuf,
    /// markdown, json, or both.
    #[arg(long, value_enum, default_value_t = ReportFormat::Markdown)]
    format: ReportFormat,
    /// Write the report files into this directory.
    #[arg(long, value_name = "DIR")]
    out_dir: Option<PathBuf>,
    /// Local SQLite database. Omit it and overlap is not checked.
    #[arg(long, value_name = "PATH")]
    db: Option<PathBuf>,
    /// Label to store or match. Only valid for a single file.
    #[arg(long)]
    label: Option<String>,
    /// Near-exact containment threshold in (0, 1]. The denominator is the query shingle set.
    #[arg(long, default_value_t = DEFAULT_MIN_CONTAINMENT)]
    min_containment: f64,
    /// Also compare against stored rows that use this source label.
    #[arg(long)]
    include_self: bool,
}

#[derive(Parser)]
struct AddArgs {
    /// File or directory of UTF-8 .txt and .md files to store.
    path: PathBuf,
    /// Local SQLite database path. This file will contain the full text.
    #[arg(long, value_name = "PATH")]
    db: PathBuf,
    /// Label for a single file. Directory entries keep their paths as labels.
    #[arg(long)]
    label: Option<String>,
    /// text or json.
    #[arg(long, value_enum, default_value_t = PlainFormat::Text)]
    format: PlainFormat,
}

#[derive(Parser)]
struct DbArgs {
    /// Local SQLite database path.
    #[arg(long, value_name = "PATH")]
    db: PathBuf,
    /// text or json.
    #[arg(long, value_enum, default_value_t = PlainFormat::Text)]
    format: PlainFormat,
}

#[derive(Clone, Copy, ValueEnum, Default)]
enum ReportFormat {
    #[default]
    Markdown,
    Json,
    Both,
}

#[derive(Clone, Copy, ValueEnum, Default)]
enum PlainFormat {
    #[default]
    Text,
    Json,
}

pub fn main<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => match dispatch(cli) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("error: {error}");
                error.exit_code()
            }
        },
        Err(error) => {
            let _ = error.print();
            error.exit_code()
        }
    }
}

fn dispatch(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Command::Scan(args) => scan(args),
        Command::Corpus(CorpusCommand::Check(args)) => {
            if args.db.is_none() {
                return Err(Error::Database {
                    path: PathBuf::from("--db"),
                    message: "corpus check requires --db PATH".to_string(),
                });
            }
            scan(args)
        }
        Command::Corpus(CorpusCommand::Add(args)) => add(args),
        Command::Corpus(CorpusCommand::List(args)) => list(args),
        Command::Corpus(CorpusCommand::Stats(args)) => stats(args),
    }
}

fn scan(args: ScanArgs) -> Result<(), Error> {
    let threshold = threshold_basis_points(args.min_containment)?;
    let loaded = load_inputs(&args.path, args.label.as_deref())?;
    let mut database = match &args.db {
        Some(path) => Some(Database::open(path)?),
        None => None,
    };
    let mut files = Vec::new();
    for file in &loaded.files {
        let corpus = if let Some(db) = database.as_mut() {
            Some(db.check_text(
                &file.label,
                &file.text,
                &CheckOptions {
                    min_containment_basis_points: threshold,
                    include_self: args.include_self,
                },
            )?)
        } else {
            None
        };
        files.push(review_file(&file.label, &file.text, file.bytes, corpus));
    }
    let report = bundle(loaded.warnings, files);
    emit_report(&args.path, args.format, args.out_dir.as_deref(), &report)
}

fn emit_report(
    input: &Path,
    format: ReportFormat,
    out_dir: Option<&Path>,
    report: &Bundle,
) -> Result<(), Error> {
    let json = render_json(report)?;
    let markdown = render_markdown(report);
    if let Some(dir) = out_dir {
        fs::create_dir_all(dir).map_err(|source| Error::Io {
            path: Some(dir.to_path_buf()),
            source,
        })?;
        let stem = output_stem(input);
        if matches!(format, ReportFormat::Markdown | ReportFormat::Both) {
            let path = dir.join(format!("{stem}.review.md"));
            write_file(&path, &markdown)?;
        }
        if matches!(format, ReportFormat::Json | ReportFormat::Both) {
            let path = dir.join(format!("{stem}.review.json"));
            write_file(&path, &json)?;
        }
    }
    match format {
        ReportFormat::Markdown => print!("{markdown}"),
        ReportFormat::Json => print!("{json}"),
        ReportFormat::Both => {
            if let Some(dir) = out_dir {
                let stem = output_stem(input);
                println!(
                    "wrote {}/{stem}.review.md and {}/{stem}.review.json",
                    dir.display(),
                    dir.display()
                );
            } else {
                print!("{markdown}\n---\n{json}");
            }
        }
    }
    Ok(())
}

fn add(args: AddArgs) -> Result<(), Error> {
    let loaded = load_inputs(&args.path, args.label.as_deref())?;
    for warning in &loaded.warnings {
        eprintln!("warning: {warning}");
    }
    let mut database = Database::open(&args.db)?;
    let mut results = Vec::new();
    for file in loaded.files {
        results.push(database.add_text(&file.label, &file.text)?);
    }
    let report = CorpusCommandReport {
        schema: CORPUS_SCHEMA,
        product: PRODUCT,
        version: env!("CARGO_PKG_VERSION"),
        database_path: args.db.display().to_string(),
        persisted_text_note: PERSISTED_TEXT_NOTE,
        disclaimer: DISCLAIMER,
        warnings: loaded.warnings,
        results,
    };
    match args.format {
        PlainFormat::Json => {
            println!("{}", serde_json_pretty(&report)?);
        }
        PlainFormat::Text => {
            println!("database: {}", report.database_path);
            for result in &report.results {
                print_add(result);
            }
            println!("note: {}", report.persisted_text_note);
        }
    }
    Ok(())
}

fn print_add(result: &AddOutcome) {
    println!("status: {}", result.status);
    println!("source_label: {}", result.source_label);
    match result.document_id {
        Some(id) => println!("document_id: {id}"),
        None => println!("document_id:"),
    }
    if let Some(existing) = &result.existing_label {
        println!("existing_label: {existing}");
    }
    println!("characters: {}", result.characters);
    println!("stored_passages: {}", result.stored_passages);
    println!("detail: {}", result.detail);
    println!();
}

fn list(args: DbArgs) -> Result<(), Error> {
    let database = Database::open(&args.db)?;
    let documents = database.list_documents()?;
    match args.format {
        PlainFormat::Json => {
            let report = ListReport {
                schema: CORPUS_SCHEMA,
                product: PRODUCT,
                version: env!("CARGO_PKG_VERSION"),
                database_path: args.db.display().to_string(),
                persisted_text_note: PERSISTED_TEXT_NOTE,
                documents,
            };
            println!("{}", serde_json_pretty(&report)?);
        }
        PlainFormat::Text => {
            println!("database: {}", args.db.display());
            println!("documents: {}", documents.len());
            for document in &documents {
                println!("id: {}", document.id);
                println!("source_label: {}", document.source_label);
                println!("content_hash: {}", document.content_hash);
                println!("characters: {}", document.characters);
                println!("passages: {}", document.passages);
                println!("added_at: {}", document.added_at);
                println!();
            }
            println!("note: {PERSISTED_TEXT_NOTE}");
            println!("Full text stays in the database and is not printed here.");
        }
    }
    Ok(())
}

fn stats(args: DbArgs) -> Result<(), Error> {
    let database = Database::open(&args.db)?;
    let coverage = database.coverage()?;
    match args.format {
        PlainFormat::Json => {
            let report = StatsReport {
                schema: CORPUS_SCHEMA,
                product: PRODUCT,
                version: env!("CARGO_PKG_VERSION"),
                database_path: args.db.display().to_string(),
                disclaimer: DISCLAIMER,
                persisted_text_note: PERSISTED_TEXT_NOTE,
                coverage,
            };
            println!("{}", serde_json_pretty(&report)?);
        }
        PlainFormat::Text => {
            println!("database: {}", args.db.display());
            println!("documents: {}", coverage.document_count);
            println!("passages: {}", coverage.passage_count);
            println!("comparable_passages: {}", coverage.comparable_passage_count);
            println!("total_characters: {}", coverage.total_characters);
            println!("scope: {}", coverage.scope);
            println!("note: {PERSISTED_TEXT_NOTE}");
            println!("{DISCLAIMER}");
        }
    }
    Ok(())
}

fn serde_json_pretty(value: &impl Serialize) -> Result<String, Error> {
    serde_json::to_string_pretty(value).map_err(|err| Error::Database {
        path: PathBuf::new(),
        message: format!("could not encode JSON: {err}"),
    })
}

fn write_file(path: &Path, contents: &str) -> Result<(), Error> {
    let mut file = fs::File::create(path).map_err(|source| Error::Io {
        path: Some(path.to_path_buf()),
        source,
    })?;
    file.write_all(contents.as_bytes())
        .map_err(|source| Error::Io {
            path: Some(path.to_path_buf()),
            source,
        })?;
    Ok(())
}

fn output_stem(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("review");
    let mut out = String::new();
    for ch in stem.chars() {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "review".to_string()
    } else {
        out
    }
}

#[derive(Serialize)]
struct CorpusCommandReport {
    schema: &'static str,
    product: &'static str,
    version: &'static str,
    database_path: String,
    persisted_text_note: &'static str,
    disclaimer: &'static str,
    warnings: Vec<String>,
    results: Vec<AddOutcome>,
}

#[derive(Serialize)]
struct ListReport {
    schema: &'static str,
    product: &'static str,
    version: &'static str,
    database_path: String,
    persisted_text_note: &'static str,
    documents: Vec<crate::corpus::DocInfo>,
}

#[derive(Serialize)]
struct StatsReport {
    schema: &'static str,
    product: &'static str,
    version: &'static str,
    database_path: String,
    disclaimer: &'static str,
    persisted_text_note: &'static str,
    coverage: crate::corpus::Coverage,
}
