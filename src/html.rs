use crate::artifacts::show_text;
use crate::report::Bundle;

pub(crate) fn render_html(bundle: &Bundle) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>");
    out.push_str(&escape(&format!(
        "{} {} precheck",
        bundle.product, bundle.version
    )));
    out.push_str("</title>\n<style>\n");
    out.push_str(STYLE);
    out.push_str("\n</style>\n</head>\n<body>\n<main>\n");
    out.push_str(&format!(
        "<p class=\"kicker\">{} {}</p>\n<h1>Coursework precheck</h1>\n",
        escape(bundle.product),
        escape(bundle.version)
    ));
    out.push_str("<aside class=\"banner\"><p>");
    out.push_str(&escape(bundle.disclaimer));
    out.push_str("</p><p>There is no overall score. Overlap numbers are 4-token shingle matches, not a reading of meaning and not a plagiarism verdict.</p></aside>\n");
    if !bundle.warnings.is_empty() {
        out.push_str("<section><h2>Warnings</h2><ul>");
        for warning in &bundle.warnings {
            out.push_str("<li>");
            out.push_str(&escape(warning));
            out.push_str("</li>");
        }
        out.push_str("</ul></section>\n");
    }
    if bundle.files.is_empty() {
        out.push_str("<p>No files were reviewed.</p>");
    }
    if bundle.files.len() > 1 {
        out.push_str("<nav class=\"toc\"><h2>Files</h2><ul>");
        for (index, file) in bundle.files.iter().enumerate() {
            let anchor = format!("file-{}", index + 1);
            out.push_str("<li><a href=\"#");
            out.push_str(&anchor);
            out.push_str("\">");
            out.push_str(&escape(&file.source_label));
            out.push_str("</a> <span class=\"quiet\">(");
            out.push_str(&escape(&file.evidence.level));
            out.push_str(")</span></li>");
        }
        out.push_str("</ul></nav>\n");
    }
    for (index, file) in bundle.files.iter().enumerate() {
        render_file(&mut out, index + 1, file);
    }
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

fn render_file(out: &mut String, file_index: usize, file: &crate::report::FileReview) {
    out.push_str(&format!("<article id=\"file-{file_index}\">\n<h2>"));
    out.push_str(&escape(&file.source_label));
    out.push_str("</h2>\n");
    out.push_str("<p class=\"meta\">");
    out.push_str(&escape(&format!(
        "Input: {}. {} bytes, {} characters. Evidence level: {}.",
        file.origin, file.input.bytes, file.input.characters, file.evidence.level
    )));
    out.push_str("</p>\n");
    out.push_str("<p>");
    out.push_str(&escape(file.evidence.authorship));
    out.push_str("</p>\n");
    if !file.extraction_notes.is_empty() {
        out.push_str("<section><h3>What was read</h3><ul>");
        for note in &file.extraction_notes {
            out.push_str("<li>");
            out.push_str(&escape(note));
            out.push_str("</li>");
        }
        out.push_str("</ul></section>\n");
    }
    render_artifacts(out, file);
    out.push_str("<section><h3>How to read the counts</h3><ul>");
    for note in &file.notes {
        out.push_str("<li>");
        out.push_str(&escape(note));
        out.push_str("</li>");
    }
    out.push_str("</ul></section>\n");
    let stats = &file.document;
    out.push_str("<section><h3>Writing surface</h3><dl>");
    item(out, "Paragraphs", &stats.paragraph_count.to_string());
    item(out, "Sentences", &stats.sentence_count.to_string());
    item(
        out,
        "Tokens",
        &format!(
            "{} words, {} Han or kana, {} numbers",
            stats.token_counts.word, stats.token_counts.cjk_or_kana, stats.token_counts.numeric
        ),
    );
    item(
        out,
        "Sentence length (min / median / max)",
        &format!(
            "{} / {} / {}",
            fmt_opt(stats.sentence_length.min),
            fmt_opt(stats.sentence_length.median),
            fmt_opt(stats.sentence_length.max)
        ),
    );
    item(
        out,
        "Mean sentence length",
        &fmt_opt(stats.sentence_length.mean),
    );
    item(
        out,
        "Sentence-length variation (stdev)",
        &fmt_opt(stats.sentence_length.sample_stdev),
    );
    item(
        out,
        "Coefficient of variation",
        &fmt_opt(stats.sentence_length.coefficient_of_variation),
    );
    item(
        out,
        "Type-token ratio",
        &fmt_opt(stats.vocabulary.type_token_ratio),
    );
    item(
        out,
        "Hapax tokens / ratio",
        &format!(
            "{} / {}",
            stats.vocabulary.hapax_count,
            fmt_opt(stats.vocabulary.hapax_ratio)
        ),
    );
    item(
        out,
        "Numeric tokens excluded from TTR",
        &stats.vocabulary.numeric_tokens_excluded.to_string(),
    );
    item(
        out,
        "Transition phrases",
        &format!(
            "{} ({} per 100 sentences)",
            stats.transitions.total,
            fmt_opt(stats.transitions.per_100_sentences)
        ),
    );
    out.push_str("</dl>");
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&stats.sentence_length.stability_note));
    out.push(' ');
    out.push_str(&escape(&stats.vocabulary.note));
    out.push_str("</p>");
    if !stats.transitions.inventory.is_empty() {
        out.push_str("<h4>Transition inventory</h4><ul>");
        for phrase in &stats.transitions.inventory {
            out.push_str("<li><code>");
            out.push_str(&escape(&phrase.phrase));
            out.push_str("</code>: ");
            out.push_str(&phrase.count.to_string());
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    out.push_str("</section>\n");
    out.push_str("<section><h3>Repetition</h3><p class=\"quiet\">");
    out.push_str(&escape(stats.repetition.note));
    out.push_str("</p><dl>");
    item(
        out,
        "Duplicate sentence types",
        &stats.repetition.duplicate_sentence_types.to_string(),
    );
    item(
        out,
        "Extra duplicate copies",
        &stats.repetition.extra_duplicate_sentences.to_string(),
    );
    out.push_str("</dl>");
    if !stats.repetition.examples.is_empty() {
        out.push_str("<h4>Duplicate sentence examples</h4><ul>");
        for example in &stats.repetition.examples {
            out.push_str("<li>");
            out.push_str(&escape(example));
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    if stats.repetition.repeated_phrases.is_empty() {
        out.push_str("<p class=\"quiet\">No 3-token phrases reached 3 occurrences.</p>");
    } else {
        out.push_str("<h4>Repeated phrases (3+ occurrences)</h4><ul>");
        for phrase in &stats.repetition.repeated_phrases {
            out.push_str("<li><code>");
            out.push_str(&escape(&phrase.phrase));
            out.push_str("</code> &times; ");
            out.push_str(&phrase.count.to_string());
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    out.push_str("</section>\n");
    out.push_str("<section><h3>Paragraphs</h3>");
    if file.paragraphs.is_empty() {
        out.push_str("<p>No paragraphs were found.</p>");
    } else {
        let open_attr = if file.paragraphs.len() <= 8 {
            " open"
        } else {
            ""
        };
        out.push_str(&format!(
            "<details{open_attr}><summary>{} paragraph(s)</summary>",
            file.paragraphs.len()
        ));
        for paragraph in &file.paragraphs {
            let location = file
                .paragraph_locations
                .get(paragraph.index - 1)
                .map(String::as_str)
                .unwrap_or("paragraph");
            out.push_str("<h4>");
            out.push_str(&escape(&format!(
                "Paragraph {} ({location})",
                paragraph.index
            )));
            out.push_str("</h4><p>");
            out.push_str(&escape(&paragraph.preview));
            out.push_str("</p><p class=\"quiet\">");
            out.push_str(&escape(&format!(
                "{} sentences, {} words / {} Han or kana / {} numeric, mean length {}, type-token ratio {}, transition hits {}, extra duplicate sentences {}.",
                paragraph.sentence_count,
                paragraph.token_counts.word,
                paragraph.token_counts.cjk_or_kana,
                paragraph.token_counts.numeric,
                fmt_opt(paragraph.sentence_length.mean),
                fmt_opt(paragraph.type_token_ratio),
                paragraph.transition_total,
                paragraph.extra_duplicate_sentences
            )));
            out.push_str("</p>");
        }
        out.push_str("</details>");
    }
    out.push_str("</section>\n");
    render_corpus(out, file);
    out.push_str("<details><summary>Method and limits</summary><ul>");
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
        file.method.no_overall_score,
        file.method.extraction,
        file.method.ocr,
        file.method.set_aside,
        file.method.database_compatibility,
        file.method.artifacts,
    ] {
        out.push_str("<li>");
        out.push_str(&escape(line));
        out.push_str("</li>");
    }
    for limitation in file.limitations {
        out.push_str("<li>");
        out.push_str(&escape(limitation));
        out.push_str("</li>");
    }
    out.push_str("</ul></details>\n</article>\n");
}

fn render_artifacts(out: &mut String, file: &crate::report::FileReview) {
    let report = &file.artifacts;
    out.push_str("<section><h3>Hidden characters and newlines</h3>");
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&format!(
        "Scope: {}. Newline style: {}.",
        report.scope, report.newline_style
    )));
    out.push_str("</p><ul>");
    for note in &report.notes {
        out.push_str("<li>");
        out.push_str(&escape(note));
        out.push_str("</li>");
    }
    out.push_str("</ul>");
    if report.findings.is_empty() {
        out.push_str("<p>No listed artifacts.</p></section>\n");
        return;
    }
    out.push_str("<ul>");
    for finding in &report.findings {
        out.push_str("<li><code>");
        out.push_str(&escape(&format!(
            "{} {} line {} column {}",
            finding.kind, finding.code_point, finding.line, finding.column
        )));
        out.push_str("</code> ");
        out.push_str(&escape(&finding.excerpt));
        out.push_str("</li>");
    }
    out.push_str("</ul>");
    if report.truncated_count > 0 {
        out.push_str("<p class=\"quiet\">");
        out.push_str(&escape(&format!(
            "{} further artifacts were omitted.",
            report.truncated_count
        )));
        out.push_str("</p>");
    }
    out.push_str("</section>\n");
}

fn render_corpus(out: &mut String, file: &crate::report::FileReview) {
    out.push_str("<section><h3>Local overlap</h3>");
    let Some(corpus) = &file.corpus else {
        out.push_str("<p>No local corpus was queried. Overlap was not checked.</p></section>\n");
        return;
    };
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&format!(
        "Database {} has {} document(s) and {} stored characters. Threshold {} basis points, at least {} shared shingles.",
        corpus.database_path,
        corpus.coverage.document_count,
        corpus.coverage.total_characters,
        corpus.min_containment_basis_points,
        corpus.min_shared_shingles
    )));
    out.push_str("</p>");
    if !corpus.notes.is_empty() {
        out.push_str("<ul>");
        for note in &corpus.notes {
            out.push_str("<li>");
            out.push_str(&escape(note));
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    if !corpus.excluded_self.is_empty() {
        out.push_str("<h4>Excluded as self</h4><ul>");
        for item in &corpus.excluded_self {
            out.push_str("<li>");
            out.push_str(&escape(&format!(
                "id {}: {} — {}",
                item.document_id, item.source_label, item.reason
            )));
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    if !corpus.document_exact_matches.is_empty() {
        out.push_str("<h4>Identical normalized documents</h4><ul>");
        for item in &corpus.document_exact_matches {
            out.push_str("<li>");
            out.push_str(&escape(&format!(
                "id {} ({}): {}",
                item.source_document_id, item.source_label, item.note
            )));
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    if corpus.matches.is_empty() {
        out.push_str(
            "<h4>Main overlaps</h4><p>No prose passage met the exact or near-exact threshold.</p>",
        );
    } else {
        out.push_str("<h4>Main overlaps</h4><p>These are shingle overlaps in passages that were not set aside. They are not a plagiarism verdict.</p>");
        for hit in &corpus.matches {
            render_hit(out, hit);
        }
        if corpus.truncated_match_count > 0 {
            out.push_str("<p class=\"quiet\">");
            out.push_str(&escape(&format!(
                "{} additional main overlaps were omitted from this report.",
                corpus.truncated_match_count
            )));
            out.push_str("</p>");
        }
    }
    if !corpus.context_overlaps.is_empty() {
        out.push_str("<h4>Set aside</h4><p>Quotations, bibliography sections, and boilerplate are listed here so you can see them. They are not treated as evidence of copied prose.</p>");
        for hit in &corpus.context_overlaps {
            render_hit(out, hit);
        }
        if corpus.truncated_context_count > 0 {
            out.push_str("<p class=\"quiet\">");
            out.push_str(&escape(&format!(
                "{} additional set-aside overlaps were omitted from this report.",
                corpus.truncated_context_count
            )));
            out.push_str("</p>");
        }
    }
    if let Some(hit) = &corpus.strongest_below_threshold {
        out.push_str("<h4>Nearest passage below the threshold</h4><p class=\"quiet\">This pair is not a match.</p>");
        render_hit(out, hit);
    }
    out.push_str("</section>\n");
}

fn render_hit(out: &mut String, hit: &crate::corpus::MatchHit) {
    out.push_str("<div class=\"hit\"><p><strong>");
    out.push_str(&escape(&format!(
        "Paragraph {} and {} paragraph {} ({})",
        hit.query_paragraph_index, hit.source_label, hit.source_paragraph_index, hit.source_kind
    )));
    out.push_str("</strong></p><p>");
    out.push_str(&escape(&format!(
        "Exact: {}. Meets threshold: {}. Containment basis points: {}. Jaccard basis points: {}. Shared shingles: {}. Query shingles (denominator): {}. Source shingles: {}.",
        if hit.exact { "yes" } else { "no" },
        if hit.meets_threshold { "yes" } else { "no" },
        hit.containment_basis_points
            .map(|points| points.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        hit.jaccard_basis_points
            .map(|points| points.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        hit.shared_shingles,
        hit.query_shingles,
        hit.source_shingles
    )));
    out.push_str("</p><p class=\"quiet\">");
    out.push_str(&escape(hit.set_aside.unwrap_or(hit.interpretation)));
    out.push(' ');
    out.push_str(&escape(hit.score_note));
    out.push_str("</p>");
    out.push_str("<blockquote><p>");
    out.push_str(&escape(&hit.query_excerpt));
    out.push_str("</p><p>");
    out.push_str(&escape(&hit.source_excerpt));
    out.push_str("</p></blockquote></div>");
}

fn item(out: &mut String, label: &str, value: &str) {
    out.push_str("<dt>");
    out.push_str(&escape(label));
    out.push_str("</dt><dd>");
    out.push_str(&escape(value));
    out.push_str("</dd>");
}

fn fmt_opt(value: Option<f64>) -> String {
    value
        .map(|number| format!("{number:.4}"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn escape(text: &str) -> String {
    let shown = show_text(text);
    let mut out = String::with_capacity(shown.len());
    for ch in shown.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c if c.is_control() && c != '\n' && c != '\t' => {}
            c => out.push(c),
        }
    }
    out
}

const STYLE: &str = "
:root { color-scheme: light; }
body { margin: 0; background: #f6f3ec; color: #1d1a16; font: 17px/1.5 Georgia, 'Songti SC', 'Noto Serif SC', serif; }
main { max-width: 42rem; margin: 0 auto; padding: 2rem 1.25rem 4rem; }
h1, h2, h3, h4 { font-family: 'Iowan Old Style', Palatino, 'Songti SC', serif; line-height: 1.25; }
.kicker { letter-spacing: 0.04em; text-transform: uppercase; font: 0.78rem/1.2 ui-sans-serif, sans-serif; color: #6a5b45; }
.banner { background: #fff8e8; border: 1px solid #e2d3b0; padding: 0.9rem 1rem; }
.quiet, .meta { color: #5c5348; font: 0.92rem/1.45 ui-sans-serif, sans-serif; }
article { margin-top: 2rem; }
dl { display: grid; grid-template-columns: 12rem 1fr; gap: 0.35rem 1rem; }
dt { font: 0.85rem/1.3 ui-sans-serif, sans-serif; color: #6a5b45; }
dd { margin: 0; }
.hit { background: white; border: 1px solid #e4dccb; padding: 0.75rem 1rem; margin: 0.75rem 0; }
blockquote { margin: 0.5rem 0 0; padding-left: 0.8rem; border-left: 3px solid #e2d3b0; }
details { margin-top: 1rem; }
.toc { margin: 1.5rem 0; padding: 0.75rem 1rem; background: #fff; border: 1px solid #e4dccb; }
.toc ul { margin: 0.4rem 0 0; padding-left: 1.25rem; }
.toc a { color: #3d3121; }
";
