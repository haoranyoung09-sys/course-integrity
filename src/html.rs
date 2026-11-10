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
    for file in &bundle.files {
        render_file(&mut out, file);
    }
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

fn render_file(out: &mut String, file: &crate::report::FileReview) {
    out.push_str("<article>\n<h2>");
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
        "Mean sentence length",
        &fmt_opt(stats.sentence_length.mean),
    );
    item(
        out,
        "Sentence-length variation",
        &fmt_opt(stats.sentence_length.sample_stdev),
    );
    item(
        out,
        "Type-token ratio",
        &fmt_opt(stats.vocabulary.type_token_ratio),
    );
    item(
        out,
        "Transition phrases",
        &stats.transitions.total.to_string(),
    );
    out.push_str("</dl>");
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&stats.sentence_length.stability_note));
    out.push(' ');
    out.push_str(&escape(&stats.vocabulary.note));
    out.push_str("</p></section>\n");
    out.push_str("<section><h3>Paragraphs</h3>");
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
            "{} sentences, mean length {}, type-token ratio {}, transition hits {}.",
            paragraph.sentence_count,
            fmt_opt(paragraph.sentence_length.mean),
            fmt_opt(paragraph.type_token_ratio),
            paragraph.transition_total
        )));
        out.push_str("</p>");
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

fn render_corpus(out: &mut String, file: &crate::report::FileReview) {
    out.push_str("<section><h3>Local overlap</h3>");
    let Some(corpus) = &file.corpus else {
        out.push_str("<p>No local corpus was queried. Overlap was not checked.</p></section>\n");
        return;
    };
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&format!(
        "Database {} has {} document(s) and {} stored characters. Threshold {} basis points.",
        corpus.database_path,
        corpus.coverage.document_count,
        corpus.coverage.total_characters,
        corpus.min_containment_basis_points
    )));
    out.push_str("</p><p>");
    out.push_str(&escape(corpus.coverage.scope));
    out.push_str("</p>");
    if corpus.matches.is_empty() {
        out.push_str(
            "<h4>Main overlaps</h4><p>No prose passage met the exact or near-exact threshold.</p>",
        );
    } else {
        out.push_str("<h4>Main overlaps</h4><p>These are shingle overlaps in passages that were not set aside. They are not a plagiarism verdict.</p>");
        for hit in &corpus.matches {
            render_hit(out, hit);
        }
    }
    if !corpus.context_overlaps.is_empty() {
        out.push_str("<h4>Set aside</h4><p>Quotations, bibliography sections, and boilerplate are listed here so you can see them. They are not treated as evidence of copied prose.</p>");
        for hit in &corpus.context_overlaps {
            render_hit(out, hit);
        }
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
        "Exact: {}. Containment basis points: {}. Shared shingles: {}. Query shingles (denominator): {}.",
        if hit.exact { "yes" } else { "no" },
        hit.containment_basis_points
            .map(|points| points.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        hit.shared_shingles,
        hit.query_shingles
    )));
    out.push_str("</p>");
    if let Some(reason) = hit.set_aside {
        out.push_str("<p>");
        out.push_str(&escape(reason));
        out.push_str("</p>");
    }
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
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
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
";
