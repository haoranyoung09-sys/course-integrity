use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run(args: &[&str]) -> std::process::Output {
    let bin = std::env::var("CARGO_BIN_EXE_course_integrity").unwrap_or_else(|_| {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push("debug");
        path.push("course-integrity");
        path.to_string_lossy().into_owned()
    });
    Command::new(bin)
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run course-integrity")
}

fn temp_db(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "course-integrity-cli-{name}-{}-{nanos}.db",
        std::process::id()
    ));
    path
}

fn json_stdout(output: &std::process::Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json stdout")
}

#[test]
fn help_lists_scan_and_corpus_without_a_risk_score() {
    let help = run(&["--help"]);
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("scan"));
    assert!(stdout.contains("corpus"));
    assert!(stdout.to_lowercase().contains("descriptive"));
    let add_help = run(&["corpus", "add", "--help"]);
    assert!(add_help.status.success());
    assert!(String::from_utf8_lossy(&add_help.stdout).contains("--db"));
    let version = run(&["--version"]);
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains("0.1.0"));
    let bare = run(&[]);
    assert_eq!(bare.status.code(), Some(2));
}

#[test]
fn scan_json_covers_empty_short_and_non_ascii() {
    let empty = json_stdout(&run(&[
        "scan",
        "fixtures/synthetic/empty.txt",
        "--format",
        "json",
    ]));
    assert_eq!(empty["schema"], "course-integrity.review.v1");
    assert!(empty["disclaimer"]
        .as_str()
        .unwrap()
        .contains("insufficient evidence for authorship"));
    assert!(empty["disclaimer"]
        .as_str()
        .unwrap()
        .contains("not a plagiarism verdict"));
    assert_eq!(empty["files"][0]["evidence"]["level"], "insufficient");
    assert!(empty["files"][0]["corpus"].is_null());
    let notes = empty["files"][0]["notes"].to_string();
    assert!(notes.contains("No local corpus was queried"));
    assert!(notes.contains("no overall score"));
    let raw = empty.to_string();
    assert!(!raw.contains("\"ai_probability\""));
    assert!(!raw.contains("\"risk_score\""));
    assert!(!raw.contains("\"confidence_interval\""));
    assert!(!raw.contains("\"overall_score\""));

    let short = json_stdout(&run(&[
        "scan",
        "fixtures/synthetic/short.txt",
        "--format",
        "json",
    ]));
    assert_eq!(short["files"][0]["evidence"]["level"], "insufficient");
    assert!(short["files"][0]["evidence"]["authorship"]
        .as_str()
        .unwrap()
        .contains("Insufficient evidence for authorship"));

    let mixed = json_stdout(&run(&[
        "scan",
        "fixtures/synthetic/non_ascii.txt",
        "--format",
        "json",
    ]));
    let counts = &mixed["files"][0]["document"]["token_counts"];
    assert!(counts["cjk_or_kana"].as_u64().unwrap() > 0);
    assert!(counts["word"].as_u64().unwrap() > 0);
    assert!(counts["numeric"].as_u64().unwrap() > 0);
    assert_eq!(mixed["files"][0]["input"]["encoding"], "utf-8");
}

#[test]
fn markdown_scan_states_the_limits() {
    let output = run(&["scan", "examples/lab-report-en.md"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Descriptive review signals only"));
    assert!(stdout.contains("not a plagiarism verdict"));
    assert!(stdout.contains("no overall score") || stdout.contains("There is no overall score"));
    assert!(stdout.contains("denominator") || stdout.contains("Containment"));
}

#[test]
fn rejects_pdf_and_invalid_utf8() {
    let dir = std::env::temp_dir().join(format!("course-integrity-cli-bad-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    let pdf = dir.join("notes.pdf");
    fs::write(&pdf, b"%PDF-1.4 synthetic").expect("pdf");
    let pdf_run = run(&["scan", pdf.to_str().expect("utf8 path")]);
    assert!(!pdf_run.status.success());
    let pdf_err = String::from_utf8_lossy(&pdf_run.stderr);
    assert!(pdf_err.contains("PDF") || pdf_err.contains("not read"));

    let bad = dir.join("bad.txt");
    fs::write(&bad, [0xff, 0xfe, 0x00]).expect("bytes");
    let bad_run = run(&["scan", bad.to_str().expect("utf8 path")]);
    assert!(!bad_run.status.success());
    assert!(String::from_utf8_lossy(&bad_run.stderr).contains("UTF-8"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cli_corpus_add_check_and_stats() {
    let db = temp_db("flow");
    let db_arg = db.to_str().expect("db path");
    let add = json_stdout(&run(&[
        "corpus",
        "add",
        "fixtures/synthetic/en_source.md",
        "--db",
        db_arg,
        "--format",
        "json",
    ]));
    assert_eq!(add["results"][0]["status"], "inserted");
    assert!(add["persisted_text_note"]
        .as_str()
        .unwrap()
        .contains("full original text"));

    let listed = json_stdout(&run(&[
        "corpus", "list", "--db", db_arg, "--format", "json",
    ]));
    assert_eq!(
        listed["documents"][0]["source_label"],
        "fixtures/synthetic/en_source.md"
    );
    assert!(
        listed["documents"][0]["content_hash"]
            .as_str()
            .unwrap()
            .len()
            >= 64
    );
    assert!(listed.get("text").is_none());

    let stats = json_stdout(&run(&[
        "corpus", "stats", "--db", db_arg, "--format", "json",
    ]));
    assert_eq!(stats["coverage"]["document_count"], 1);
    assert!(stats["coverage"]["scope"]
        .as_str()
        .unwrap()
        .contains("Turnitin"));

    let check = json_stdout(&run(&[
        "corpus",
        "check",
        "fixtures/synthetic/en_exact_paragraph.md",
        "--db",
        db_arg,
        "--format",
        "json",
    ]));
    let matches = check["files"][0]["corpus"]["matches"]
        .as_array()
        .expect("matches");
    assert!(matches
        .iter()
        .any(|hit| hit["exact"].as_bool() == Some(true)));
    assert!(matches
        .iter()
        .all(|hit| hit["score_note"].as_str().unwrap().contains("denominator")));

    let again = json_stdout(&run(&[
        "corpus",
        "add",
        "fixtures/synthetic/en_source.md",
        "--db",
        db_arg,
        "--format",
        "json",
    ]));
    assert_eq!(again["results"][0]["status"], "duplicate");

    let self_check = json_stdout(&run(&[
        "scan",
        "fixtures/synthetic/en_source.md",
        "--db",
        db_arg,
        "--format",
        "json",
    ]));
    assert!(self_check["files"][0]["corpus"]["matches"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!self_check["files"][0]["corpus"]["excluded_self"]
        .as_array()
        .unwrap()
        .is_empty());

    drop_db(&db);
}

fn drop_db(path: &std::path::Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("db-journal"));
}

#[test]
fn directory_warns_about_pdf_and_still_scans_text() {
    let dir = std::env::temp_dir().join(format!("course-integrity-cli-dir-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("dir");
    fs::write(dir.join("note.txt"), "Short note.\n").expect("txt");
    fs::write(dir.join("slides.pdf"), b"%PDF synthetic").expect("pdf");
    let output = json_stdout(&run(&[
        "scan",
        dir.to_str().expect("dir"),
        "--format",
        "json",
    ]));
    assert_eq!(output["files"].as_array().unwrap().len(), 1);
    assert!(
        output["warnings"].to_string().contains("PDF")
            || output["warnings"].to_string().contains("not read")
    );
    let _ = fs::remove_dir_all(&dir);
}
