use serde::Serialize;

use crate::corpus::CorpusSection;
use crate::method::{
    CONTAINMENT_DENOMINATOR, COUNT_STREAMS, DISCLAIMER, EXACT_DESC, HASH_DESC, JACCARD_DESC,
    LENGTH_UNIT_NOTE, LIMITATIONS, NOT_QUERIED_LINE, NO_OVERALL_SCORE, PRODUCT, REPETITION_NOTE,
    SCHEMA, SELF_DESC, SHINGLE_DESC, TOKENIZER_DESC, TRANSITION_NOTE, UNICODE_DESC, VOCAB_NOTE,
};
use crate::stats::{analyze, DocumentStats, Evidence, ParagraphStats};

#[derive(Debug, Clone, Serialize)]
pub struct InputInfo {
    pub bytes: u64,
    pub characters: usize,
    pub encoding: &'static str,
    pub bom_stripped: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodStamp {
    pub tokenizer: &'static str,
    pub count_streams: &'static str,
    pub shingles: &'static str,
    pub containment_denominator: &'static str,
    pub jaccard: &'static str,
    pub unicode: &'static str,
    pub exact_match: &'static str,
    pub document_hash: &'static str,
    pub self_comparison: &'static str,
    pub sentence_length: &'static str,
    pub vocabulary: &'static str,
    pub repetition: &'static str,
    pub transitions: &'static str,
    pub no_overall_score: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileReview {
    pub source_label: String,
    pub input: InputInfo,
    pub evidence: Evidence,
    pub notes: Vec<String>,
    pub method: MethodStamp,
    pub limitations: &'static [&'static str],
    pub document: DocumentStats,
    pub paragraphs: Vec<ParagraphStats>,
    pub corpus: Option<CorpusSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bundle {
    pub schema: &'static str,
    pub product: &'static str,
    pub version: &'static str,
    pub disclaimer: &'static str,
    pub warnings: Vec<String>,
    pub files: Vec<FileReview>,
}

pub(crate) fn review_file(
    label: &str,
    text: &str,
    bytes: u64,
    corpus: Option<CorpusSection>,
) -> FileReview {
    let analysis = analyze(text);
    let mut notes = analysis.notes;
    if corpus.is_none() {
        notes.push(NOT_QUERIED_LINE.to_string());
    }
    FileReview {
        source_label: label.to_string(),
        input: InputInfo {
            bytes,
            characters: analysis.characters,
            encoding: "utf-8",
            bom_stripped: analysis.bom_stripped,
        },
        evidence: analysis.evidence,
        notes,
        method: method_stamp(),
        limitations: LIMITATIONS,
        document: analysis.document,
        paragraphs: analysis.paragraphs,
        corpus,
    }
}

pub(crate) fn bundle(warnings: Vec<String>, files: Vec<FileReview>) -> Bundle {
    Bundle {
        schema: SCHEMA,
        product: PRODUCT,
        version: env!("CARGO_PKG_VERSION"),
        disclaimer: DISCLAIMER,
        warnings,
        files,
    }
}

fn method_stamp() -> MethodStamp {
    MethodStamp {
        tokenizer: TOKENIZER_DESC,
        count_streams: COUNT_STREAMS,
        shingles: SHINGLE_DESC,
        containment_denominator: CONTAINMENT_DENOMINATOR,
        jaccard: JACCARD_DESC,
        unicode: UNICODE_DESC,
        exact_match: EXACT_DESC,
        document_hash: HASH_DESC,
        self_comparison: SELF_DESC,
        sentence_length: LENGTH_UNIT_NOTE,
        vocabulary: VOCAB_NOTE,
        repetition: REPETITION_NOTE,
        transitions: TRANSITION_NOTE,
        no_overall_score: NO_OVERALL_SCORE,
    }
}

pub(crate) fn render_json(bundle: &Bundle) -> Result<String, crate::error::Error> {
    let mut json =
        serde_json::to_string_pretty(bundle).map_err(|err| crate::error::Error::Database {
            path: std::path::PathBuf::new(),
            message: format!("could not encode the report: {err}"),
        })?;
    json.push('\n');
    Ok(json)
}

pub(crate) fn render_markdown(bundle: &Bundle) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# {} {} precheck\n\n",
        bundle.product, bundle.version
    ));
    out.push_str(&format!("> {}\n\n", bundle.disclaimer));
    out.push_str("There is no overall score in this report.\n\n");
    if !bundle.warnings.is_empty() {
        out.push_str("## Warnings\n\n");
        for warning in &bundle.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
        out.push('\n');
    }
    if bundle.files.is_empty() {
        out.push_str("No files were reviewed.\n");
        return out;
    }
    for file in &bundle.files {
        render_file(&mut out, file);
    }
    out
}

fn render_file(out: &mut String, file: &FileReview) {
    out.push_str(&format!("## {}\n\n", file.source_label));
    out.push_str(&format!(
        "- Encoding: {}\n- Bytes: {}\n- Characters: {}\n- BOM stripped: {}\n\n",
        file.input.encoding,
        file.input.bytes,
        file.input.characters,
        if file.input.bom_stripped { "yes" } else { "no" }
    ));
    out.push_str("### Evidence limits\n\n");
    out.push_str(&format!("- Level: `{}`\n", file.evidence.level));
    out.push_str(&format!("- Authorship: {}\n", file.evidence.authorship));
    for reason in &file.evidence.reasons {
        out.push_str(&format!("- {reason}\n"));
    }
    out.push('\n');
    out.push_str("### Notes\n\n");
    for note in &file.notes {
        out.push_str(&format!("- {note}\n"));
    }
    out.push('\n');
    render_document(out, &file.document);
    out.push_str("### Paragraphs\n\n");
    if file.paragraphs.is_empty() {
        out.push_str("No paragraphs were found.\n\n");
    }
    for paragraph in &file.paragraphs {
        out.push_str(&format!("#### Paragraph {}\n\n", paragraph.index));
        out.push_str(&format!("{}\n\n", paragraph.preview));
        out.push_str(&format!(
            "- Sentences: {}\n- Tokens: {} words, {} Han or kana, {} numeric\n- Mean sentence length: {}\n- Type-token ratio: {}\n- Transition hits: {}\n- Extra duplicate sentences: {}\n\n",
            paragraph.sentence_count,
            paragraph.token_counts.word,
            paragraph.token_counts.cjk_or_kana,
            paragraph.token_counts.numeric,
            fmt_opt(paragraph.sentence_length.mean),
            fmt_opt(paragraph.type_token_ratio),
            paragraph.transition_total,
            paragraph.extra_duplicate_sentences
        ));
    }
    render_corpus(out, &file.corpus);
    out.push_str("### Method\n\n");
    for line in [
        file.method.tokenizer,
        file.method.count_streams,
        file.method.shingles,
        file.method.containment_denominator,
        file.method.jaccard,
        file.method.unicode,
        file.method.exact_match,
        file.method.document_hash,
        file.method.self_comparison,
        file.method.sentence_length,
        file.method.vocabulary,
        file.method.repetition,
        file.method.transitions,
        file.method.no_overall_score,
    ] {
        out.push_str(&format!("- {line}\n"));
    }
    out.push_str("\n### Limitations\n\n");
    for limitation in file.limitations {
        out.push_str(&format!("- {limitation}\n"));
    }
    out.push('\n');
}

fn render_document(out: &mut String, document: &DocumentStats) {
    let length = &document.sentence_length;
    out.push_str("### Document statistics\n\n");
    out.push_str(&format!(
        "- Paragraphs: {}\n- Sentences: {}\n- Tokens: {} words, {} Han or kana, {} numeric\n\n",
        document.paragraph_count,
        document.sentence_count,
        document.token_counts.word,
        document.token_counts.cjk_or_kana,
        document.token_counts.numeric
    ));
    out.push_str("#### Sentence length\n\n");
    out.push_str(&format!("{}\n\n", document.sentence_length_unit_note));
    out.push_str(&format!(
        "| Measure | Value |\n| --- | --- |\n| unit | {} |\n| count | {} |\n| min | {} |\n| max | {} |\n| mean | {} |\n| median | {} |\n| sample standard deviation | {} |\n| coefficient of variation | {} |\n| stable | {} |\n\n{}\n\n",
        length.unit,
        length.count,
        fmt_opt(length.min),
        fmt_opt(length.max),
        fmt_opt(length.mean),
        fmt_opt(length.median),
        fmt_opt(length.sample_stdev),
        fmt_opt(length.coefficient_of_variation),
        if length.stable { "yes" } else { "no" },
        length.stability_note
    ));
    let repetition = &document.repetition;
    out.push_str("#### Repetition\n\n");
    out.push_str(&format!("{}\n\n", repetition.note));
    out.push_str(&format!(
        "- Duplicate sentence types: {}\n- Extra copies: {}\n",
        repetition.duplicate_sentence_types, repetition.extra_duplicate_sentences
    ));
    if repetition.examples.is_empty() {
        out.push_str("- Examples: none\n");
    } else {
        for example in &repetition.examples {
            out.push_str(&format!("- Example: {example}\n"));
        }
    }
    if repetition.repeated_phrases.is_empty() {
        out.push_str("- Repeated phrases: none reached 3 occurrences\n\n");
    } else {
        out.push('\n');
        for phrase in &repetition.repeated_phrases {
            out.push_str(&format!("- `{}` × {}\n", phrase.phrase, phrase.count));
        }
        out.push('\n');
    }
    let vocabulary = &document.vocabulary;
    out.push_str("#### Vocabulary\n\n");
    out.push_str(&format!(
        "- Non-numeric tokens: {}\n- Unique: {}\n- Type-token ratio: {}\n- Hapax tokens: {}\n- Hapax ratio: {}\n- Numeric tokens excluded: {}\n- Stable: {}\n\n{}\n\n",
        vocabulary.non_numeric_tokens,
        vocabulary.unique_non_numeric_tokens,
        fmt_opt(vocabulary.type_token_ratio),
        vocabulary.hapax_count,
        fmt_opt(vocabulary.hapax_ratio),
        vocabulary.numeric_tokens_excluded,
        if vocabulary.stable { "yes" } else { "no" },
        vocabulary.note
    ));
    let transitions = &document.transitions;
    out.push_str("#### Transition inventory\n\n");
    out.push_str(&format!("{}\n\n", transitions.note));
    out.push_str(&format!(
        "- Total hits: {}\n- Per 100 sentences: {}\n",
        transitions.total,
        fmt_opt(transitions.per_100_sentences)
    ));
    if transitions.inventory.is_empty() {
        out.push_str("- None of the listed phrases appeared.\n\n");
    } else {
        out.push('\n');
        for phrase in &transitions.inventory {
            out.push_str(&format!("- `{}`: {}\n", phrase.phrase, phrase.count));
        }
        out.push('\n');
    }
}

fn render_corpus(out: &mut String, corpus: &Option<CorpusSection>) {
    out.push_str("### Local corpus\n\n");
    let Some(corpus) = corpus else {
        out.push_str(&format!("{NOT_QUERIED_LINE}\n\n"));
        return;
    };
    out.push_str(&format!("- Database: {}\n", corpus.database_path));
    out.push_str(&format!(
        "- Documents: {}\n- Passages: {}\n- Near-exact comparable passages: {}\n- Stored characters: {}\n- Threshold: {} basis points, at least {} shared shingles\n\n",
        corpus.coverage.document_count,
        corpus.coverage.passage_count,
        corpus.coverage.comparable_passage_count,
        corpus.coverage.total_characters,
        corpus.min_containment_basis_points,
        corpus.min_shared_shingles
    ));
    out.push_str(&format!("{}\n\n", corpus.coverage.scope));
    out.push_str(&format!("{}\n\n", corpus.persisted_text_note));
    for note in &corpus.notes {
        out.push_str(&format!("- {note}\n"));
    }
    out.push('\n');
    if corpus.excluded_self.is_empty() {
        out.push_str("No stored document was excluded as the same source label.\n\n");
    } else {
        out.push_str("#### Excluded as self\n\n");
        for item in &corpus.excluded_self {
            out.push_str(&format!(
                "- id {}: {} — {}\n",
                item.document_id, item.source_label, item.reason
            ));
        }
        out.push('\n');
    }
    if !corpus.document_exact_matches.is_empty() {
        out.push_str("#### Identical normalized documents\n\n");
        for item in &corpus.document_exact_matches {
            out.push_str(&format!(
                "- id {} ({})\n\n{}\n\n",
                item.source_document_id, item.source_label, item.note
            ));
        }
    }
    if corpus.matches.is_empty() {
        out.push_str("#### Matches\n\nNo passage met the exact or near-exact threshold.\n\n");
    } else {
        out.push_str("#### Matches\n\n");
        out.push_str(
            "These hits are descriptive overlap signals. They are not a plagiarism verdict.\n\n",
        );
        for hit in &corpus.matches {
            render_hit(out, hit);
        }
        if corpus.truncated_match_count > 0 {
            out.push_str(&format!(
                "{} additional matches were omitted from this report.\n\n",
                corpus.truncated_match_count
            ));
        }
    }
    if let Some(hit) = &corpus.strongest_below_threshold {
        out.push_str("#### Nearest passage below the threshold\n\n");
        out.push_str("This pair is not a match.\n\n");
        render_hit(out, hit);
    }
}

fn render_hit(out: &mut String, hit: &crate::corpus::MatchHit) {
    out.push_str(&format!(
        "- Query paragraph {} ↔ document {} paragraph {} (`{}`)\n",
        hit.query_paragraph_index,
        hit.source_document_id,
        hit.source_paragraph_index,
        hit.source_label
    ));
    out.push_str(&format!(
        "  - Exact: {}\n  - Meets threshold: {}\n  - Shared shingles: {}\n  - Query shingles (denominator): {}\n  - Source shingles: {}\n  - Containment basis points: {}\n  - Jaccard basis points: {}\n  - {}\n  - {}\n  - Query excerpt: {}\n  - Source excerpt: {}\n\n",
        if hit.exact { "yes" } else { "no" },
        if hit.meets_threshold { "yes" } else { "no" },
        hit.shared_shingles,
        hit.query_shingles,
        hit.source_shingles,
        fmt_points(hit.containment_basis_points),
        fmt_points(hit.jaccard_basis_points),
        hit.score_note,
        hit.interpretation,
        hit.query_excerpt,
        hit.source_excerpt
    ));
}

fn fmt_opt(value: Option<f64>) -> String {
    value
        .map(|number| format!("{number:.4}"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn fmt_points(value: Option<u32>) -> String {
    value
        .map(|points| points.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}
