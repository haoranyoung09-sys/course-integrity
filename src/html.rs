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
    out.push_str("\n</style>\n</head>\n<body>\n<main id=\"top\">\n");
    out.push_str(&format!(
        "<header class=\"page-head\"><p class=\"kicker\">{} {}</p>\n<h1>Coursework precheck</h1></header>\n",
        escape(bundle.product),
        escape(bundle.version)
    ));
    out.push_str("<aside class=\"banner\"><p>");
    out.push_str(&escape(bundle.disclaimer));
    out.push_str("</p><p>There is no overall score. Overlap numbers are 4-token shingle matches, not a reading of meaning and not a plagiarism verdict.</p></aside>\n");
    if !bundle.warnings.is_empty() {
        out.push_str("<section class=\"warnings\"><h2>Warnings</h2><ul>");
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
        render_toc(&mut out, bundle);
    }
    for (index, file) in bundle.files.iter().enumerate() {
        render_file(&mut out, index + 1, bundle.files.len() > 1, file);
    }
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

fn render_toc(out: &mut String, bundle: &Bundle) {
    out.push_str("<nav class=\"toc\" aria-label=\"Reviewed files\"><h2>Files (");
    out.push_str(&bundle.files.len().to_string());
    out.push_str(")</h2><div class=\"table-wrap\"><table><thead><tr><th>File</th><th>Origin</th><th>Evidence</th><th>Sentences / Tokens</th><th>Artifacts</th><th>Local overlap</th></tr></thead><tbody>");
    for (index, file) in bundle.files.iter().enumerate() {
        let anchor = format!("file-{}", index + 1);
        let tokens = file.document.token_counts.word
            + file.document.token_counts.cjk_or_kana
            + file.document.token_counts.numeric;
        let artifacts_total = file.artifacts.findings.len() + file.artifacts.truncated_count;
        let overlap_cell = match &file.corpus {
            None => "not queried".to_string(),
            Some(corpus) => {
                let mut text = format!(
                    "{} main / {} set aside",
                    corpus.matches.len(),
                    corpus.context_overlaps.len()
                );
                if !corpus.document_exact_matches.is_empty() {
                    text.push_str(&format!(
                        " ({} doc-exact)",
                        corpus.document_exact_matches.len()
                    ));
                }
                text
            }
        };
        out.push_str("<tr><td><a href=\"#");
        out.push_str(&anchor);
        out.push_str("\">");
        out.push_str(&escape(&file.source_label));
        out.push_str("</a></td><td><code>");
        out.push_str(&escape(&file.origin));
        out.push_str("</code></td><td><code>");
        out.push_str(&escape(&file.evidence.level));
        out.push_str("</code></td><td>");
        out.push_str(&escape(&format!(
            "{} / {}",
            file.document.sentence_count, tokens
        )));
        out.push_str("</td><td>");
        out.push_str(&artifacts_total.to_string());
        out.push_str("</td><td>");
        out.push_str(&escape(&overlap_cell));
        out.push_str("</td></tr>");
    }
    out.push_str("</tbody></table></div></nav>\n");
}

fn render_file(
    out: &mut String,
    file_index: usize,
    multi_file: bool,
    file: &crate::report::FileReview,
) {
    let prefix = format!("file-{file_index}");
    let stats = &file.document;
    let total_tokens =
        stats.token_counts.word + stats.token_counts.cjk_or_kana + stats.token_counts.numeric;
    let artifacts_total = file.artifacts.findings.len() + file.artifacts.truncated_count;

    out.push_str(&format!(
        "<article id=\"{prefix}\">\n<header class=\"file-head\"><h2>"
    ));
    out.push_str(&escape(&file.source_label));
    out.push_str("</h2>\n<p class=\"meta\">");
    out.push_str(&escape(&format!(
        "Input: {}. {} bytes, {} characters. Evidence level: {}.",
        file.origin, file.input.bytes, file.input.characters, file.evidence.level
    )));
    out.push_str("</p>\n<p class=\"quiet\">");
    out.push_str(&escape(file.evidence.authorship));
    out.push_str("</p>\n");

    out.push_str("<div class=\"cards\">");
    card(
        out,
        "Evidence level",
        &file.evidence.level,
        "Stability of counts only, not authorship",
    );
    card(
        out,
        "Volume",
        &format!("{} sentences", stats.sentence_count),
        &format!(
            "{} paragraphs · {} tokens",
            stats.paragraph_count, total_tokens
        ),
    );
    card(
        out,
        "Hidden artifacts",
        &format!("{artifacts_total} listed"),
        &format!("Newlines: {}", file.artifacts.newline_style),
    );
    match &file.corpus {
        Some(corpus) => {
            let main_label = format!(
                "{} main · {} set aside",
                corpus.matches.len(),
                corpus.context_overlaps.len()
            );
            let sub = if !corpus.document_exact_matches.is_empty() {
                format!(
                    "{} identical doc(s) · threshold {} bp",
                    corpus.document_exact_matches.len(),
                    corpus.min_containment_basis_points
                )
            } else {
                format!(
                    "{} doc(s) in scope · threshold {} bp",
                    corpus.coverage.document_count, corpus.min_containment_basis_points
                )
            };
            card(out, "Local overlap", &main_label, &sub);
        }
        None => {
            card(
                out,
                "Local overlap",
                "Not queried",
                "Pass --db or --pairwise to compare",
            );
        }
    }
<<<<<<< Updated upstream
    render_artifacts(out, file);
    render_ai(out, file);
    out.push_str("<section><h3>How to read the counts</h3><ul>");
    for note in &file.notes {
        out.push_str("<li>");
        out.push_str(&escape(note));
        out.push_str("</li>");
=======
    out.push_str("</div>\n");

    out.push_str("<nav class=\"jump\" aria-label=\"Sections\">");
    jump_link(out, &format!("#{prefix}-overlap"), "Local overlap");
    jump_link(out, &format!("#{prefix}-surface"), "Writing surface");
    jump_link(out, &format!("#{prefix}-repetition"), "Repetition");
    jump_link(out, &format!("#{prefix}-artifacts"), "Hidden characters");
    jump_link(out, &format!("#{prefix}-paragraphs"), "Paragraphs");
    jump_link(out, &format!("#{prefix}-method"), "Method &amp; limits");
    if multi_file {
        jump_link(out, "#top", "All files");
>>>>>>> Stashed changes
    }
    out.push_str("</nav></header>\n");

    render_corpus(out, &prefix, file);

    out.push_str(&format!(
        "<section id=\"{prefix}-surface\"><h3>Writing surface</h3><dl>"
    ));
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
        out.push_str("<h4>Transition inventory</h4><ul class=\"pills\">");
        for phrase in &stats.transitions.inventory {
            out.push_str("<li><code>");
            out.push_str(&escape(&phrase.phrase));
            out.push_str("</code> &times; ");
            out.push_str(&phrase.count.to_string());
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    out.push_str("</section>\n");

    out.push_str(&format!(
        "<section id=\"{prefix}-repetition\"><h3>Repetition</h3><p class=\"quiet\">"
    ));
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
        out.push_str("<h4>Repeated phrases (3+ occurrences)</h4><ul class=\"pills\">");
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

    render_artifacts(out, &prefix, file);

    out.push_str(&format!(
        "<section id=\"{prefix}-paragraphs\"><h3>Paragraphs</h3>"
    ));
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
            out.push_str(&format!(
                "<div class=\"para\" id=\"{prefix}-p-{}\"><h4>",
                paragraph.index
            ));
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
            out.push_str("</p></div>");
        }
        out.push_str("</details>");
    }
    out.push_str("</section>\n");

    if !file.extraction_notes.is_empty() {
        out.push_str("<details><summary>What was read</summary><ul>");
        for note in &file.extraction_notes {
            out.push_str("<li>");
            out.push_str(&escape(note));
            out.push_str("</li>");
        }
        out.push_str("</ul></details>\n");
    }
    if !file.notes.is_empty() {
        out.push_str("<details><summary>How to read the counts</summary><ul>");
        for note in &file.notes {
            out.push_str("<li>");
            out.push_str(&escape(note));
            out.push_str("</li>");
        }
        out.push_str("</ul></details>\n");
    }
    out.push_str(&format!(
        "<details id=\"{prefix}-method\"><summary>Method and limits</summary><ul>"
    ));
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

fn render_artifacts(out: &mut String, prefix: &str, file: &crate::report::FileReview) {
    let report = &file.artifacts;
    out.push_str(&format!(
        "<section id=\"{prefix}-artifacts\"><h3>Hidden characters and newlines</h3>"
    ));
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&format!(
        "Scope: {}. Newline style: {}.",
        report.scope, report.newline_style
    )));
    out.push_str("</p>");
    if report.findings.is_empty() {
        out.push_str("<p>No listed artifacts.</p>");
    } else {
        out.push_str("<div class=\"table-wrap\"><table><thead><tr><th>Kind</th><th>Code point</th><th>Line : Col</th><th>Excerpt</th></tr></thead><tbody>");
        for finding in &report.findings {
            out.push_str("<tr><td><code>");
            out.push_str(&escape(&finding.kind));
            out.push_str("</code></td><td><code>");
            out.push_str(&escape(&finding.code_point));
            out.push_str("</code></td><td><code>");
            out.push_str(&format!("{}:{}", finding.line, finding.column));
            out.push_str("</code></td><td class=\"excerpt-cell\">");
            out.push_str(&escape(&finding.excerpt));
            out.push_str("</td></tr>");
        }
        out.push_str("</tbody></table></div>");
    }
    if report.truncated_count > 0 {
        out.push_str("<p class=\"quiet\">");
        out.push_str(&escape(&format!(
            "{} further artifacts were omitted.",
            report.truncated_count
        )));
        out.push_str("</p>");
    }
    out.push_str("<details><summary>Artifact inventory notes</summary><ul>");
    for note in &report.notes {
        out.push_str("<li>");
        out.push_str(&escape(note));
        out.push_str("</li>");
    }
    out.push_str("</ul></details></section>\n");
}

<<<<<<< Updated upstream
fn render_ai(out: &mut String, file: &crate::report::FileReview) {
    let ai = &file.ai_classifier;
    out.push_str("<section><h3>Optional local classifier</h3>");
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&format!("Status: {}.", ai.status)));
    if let Some(reason) = &ai.reason {
        out.push(' ');
        out.push_str(&escape(reason));
    }
    out.push_str("</p>");
    if ai.status != "ok" {
        out.push_str("<p>");
        out.push_str(&escape(ai.not_a_zero_result));
        out.push_str("</p>");
    }
    for chunk in &ai.chunks {
        out.push_str("<div class=\"hit\"><p><strong>");
        out.push_str(&escape(&format!(
            "{} · {} · characters {}–{}",
            chunk.location, chunk.language, chunk.char_start, chunk.char_end
        )));
        out.push_str("</strong></p>");
        if let Some(score) = chunk.score_0_to_100 {
            out.push_str("<p><strong>");
            out.push_str(&escape(ai.label));
            out.push_str(": ");
            out.push_str(&escape(&format!("{score}")));
            out.push_str("</strong></p><p>");
            out.push_str(&escape(ai.distinction));
            out.push_str("</p><p class=\"quiet\">");
            out.push_str(&escape(&format!(
                "Predicted class: {}. Tokens: {} of 512.",
                chunk.predicted_class.as_deref().unwrap_or("unknown"),
                chunk.token_count.unwrap_or(0)
            )));
            out.push_str("</p>");
        } else {
            out.push_str("<p>Not scored: ");
            out.push_str(&escape(chunk.reason.as_deref().unwrap_or("not_scored")));
            out.push_str("</p>");
        }
        out.push_str("</div>");
    }
    if let Some(share) = ai.flagged_character_share {
        out.push_str("<p>");
        out.push_str(&escape(&format!(
            "Classifier-flagged character share: {share} ({}/{} scored characters). {}",
            ai.flagged_characters, ai.scored_characters, ai.flagged_share_definition
        )));
        out.push_str("</p>");
    }
    out.push_str("<p class=\"quiet\">");
    out.push_str(&escape(&format!(
        "Coverage: {} scored, {} skipped, {} not covered by a tokenizer window.",
        ai.scored_characters, ai.skipped_characters, ai.uncovered_characters
    )));
    out.push_str("</p></section>\n");
}

fn render_corpus(out: &mut String, file: &crate::report::FileReview) {
    out.push_str("<section><h3>Local overlap</h3>");
=======
fn render_corpus(out: &mut String, prefix: &str, file: &crate::report::FileReview) {
    out.push_str(&format!(
        "<section id=\"{prefix}-overlap\"><h3>Local overlap</h3>"
    ));
>>>>>>> Stashed changes
    let Some(corpus) = &file.corpus else {
        out.push_str("<p class=\"quiet\">No local corpus was queried. Overlap was not checked.</p></section>\n");
        return;
    };
    let is_pairwise = corpus.database_path.starts_with("in-memory pairwise");
    out.push_str("<p class=\"quiet\">");
    if is_pairwise {
        out.push_str(&escape(&format!(
            "Comparison mode: {}. {} comparable file(s) ({} characters). Threshold {} ({} basis points), at least {} shared shingles.",
            corpus.database_path,
            corpus.coverage.document_count,
            corpus.coverage.total_characters,
            fmt_bp_pct(Some(corpus.min_containment_basis_points)),
            corpus.min_containment_basis_points,
            corpus.min_shared_shingles
        )));
    } else {
        out.push_str(&escape(&format!(
            "Database {} has {} document(s) and {} stored characters. Threshold {} ({} basis points), at least {} shared shingles.",
            corpus.database_path,
            corpus.coverage.document_count,
            corpus.coverage.total_characters,
            fmt_bp_pct(Some(corpus.min_containment_basis_points)),
            corpus.min_containment_basis_points,
            corpus.min_shared_shingles
        )));
    }
    out.push_str("</p>");
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
            "<h4>Main overlaps</h4><p class=\"quiet\">No prose passage met the exact or near-exact threshold.</p>",
        );
    } else {
        out.push_str("<h4>Main overlaps</h4><p class=\"quiet\">These are shingle overlaps in passages that were not set aside. They are not a plagiarism verdict.</p>");
        for hit in &corpus.matches {
            render_hit(out, prefix, hit);
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
        out.push_str("<h4>Set aside</h4><p class=\"quiet\">Quotations, bibliography sections, and boilerplate are listed here so you can see them. They are not treated as evidence of copied prose.</p>");
        for hit in &corpus.context_overlaps {
            render_hit(out, prefix, hit);
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
        render_hit(out, prefix, hit);
    }
    if !corpus.excluded_self.is_empty() {
        out.push_str("<details><summary>Excluded as self (");
        out.push_str(&corpus.excluded_self.len().to_string());
        out.push_str(")</summary><ul>");
        for item in &corpus.excluded_self {
            out.push_str("<li>");
            out.push_str(&escape(&format!(
                "id {}: {} — {}",
                item.document_id, item.source_label, item.reason
            )));
            out.push_str("</li>");
        }
        out.push_str("</ul></details>");
    }
    if !corpus.notes.is_empty() {
        out.push_str("<details><summary>Corpus scope and notes</summary><ul>");
        for note in &corpus.notes {
            out.push_str("<li>");
            out.push_str(&escape(note));
            out.push_str("</li>");
        }
        out.push_str("</ul></details>");
    }
    out.push_str("</section>\n");
}

fn render_hit(out: &mut String, prefix: &str, hit: &crate::corpus::MatchHit) {
    out.push_str("<div class=\"hit\"><div class=\"hit-head\"><strong>");
    out.push_str(&format!(
        "<a href=\"#{prefix}-p-{}\">Paragraph {}</a>",
        hit.query_paragraph_index, hit.query_paragraph_index
    ));
    out.push_str(" &harr; ");
    out.push_str(&escape(&format!(
        "{} paragraph {} ({})",
        hit.source_label, hit.source_paragraph_index, hit.source_kind
    )));
    out.push_str("</strong></div>");

    out.push_str("<div class=\"hit-metrics\">");
    metric_badge(out, "Exact", if hit.exact { "yes" } else { "no" });
    metric_badge(
        out,
        "Meets threshold",
        if hit.meets_threshold { "yes" } else { "no" },
    );
    metric_badge(
        out,
        "Containment",
        &format!(
            "{} ({} bp)",
            fmt_bp_pct(hit.containment_basis_points),
            hit.containment_basis_points
                .map(|points| points.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        ),
    );
    metric_badge(
        out,
        "Jaccard",
        &format!(
            "{} ({} bp)",
            fmt_bp_pct(hit.jaccard_basis_points),
            hit.jaccard_basis_points
                .map(|points| points.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        ),
    );
    metric_badge(
        out,
        "Shared shingles",
        &format!(
            "{} / {} query ({} source)",
            hit.shared_shingles, hit.query_shingles, hit.source_shingles
        ),
    );
    out.push_str("</div>");

    out.push_str("<div class=\"compare\"><div class=\"pane\"><p class=\"pane-label\">");
    out.push_str(&escape(&format!(
        "Draft — Paragraph {}",
        hit.query_paragraph_index
    )));
    out.push_str("</p><blockquote><p>");
    out.push_str(&escape(&hit.query_excerpt));
    out.push_str("</p></blockquote></div><div class=\"pane\"><p class=\"pane-label\">");
    out.push_str(&escape(&format!(
        "Source — {} (Paragraph {})",
        hit.source_label, hit.source_paragraph_index
    )));
    out.push_str("</p><blockquote><p>");
    out.push_str(&escape(&hit.source_excerpt));
    out.push_str("</p></blockquote></div></div>");

    out.push_str("<p class=\"quiet hit-note\">");
    out.push_str(&escape(hit.set_aside.unwrap_or(hit.interpretation)));
    out.push(' ');
    out.push_str(&escape(hit.score_note));
    out.push_str("</p></div>");
}

fn card(out: &mut String, label: &str, value: &str, sub: &str) {
    out.push_str("<div class=\"card\"><div class=\"card-label\">");
    out.push_str(&escape(label));
    out.push_str("</div><div class=\"card-value\">");
    out.push_str(&escape(value));
    out.push_str("</div><div class=\"card-sub\">");
    out.push_str(&escape(sub));
    out.push_str("</div></div>");
}

fn jump_link(out: &mut String, href: &str, label: &str) {
    out.push_str("<a href=\"");
    out.push_str(href);
    out.push_str("\">");
    out.push_str(label);
    out.push_str("</a>");
}

fn metric_badge(out: &mut String, label: &str, value: &str) {
    out.push_str("<span class=\"badge\"><b>");
    out.push_str(&escape(label));
    out.push_str(":</b> ");
    out.push_str(&escape(value));
    out.push_str("</span>");
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

fn fmt_bp_pct(points: Option<u32>) -> String {
    points
        .map(|bp| format!("{:.2}%", f64::from(bp) / 100.0))
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
:root {
  color-scheme: light dark;
  --bg: #f6f3ec;
  --surface: #ffffff;
  --surface-alt: #fbf9f4;
  --banner-bg: #fff8e8;
  --banner-border: #e2d3b0;
  --border: #e4dccb;
  --text: #1d1a16;
  --muted: #5c5348;
  --accent: #6a5b45;
  --link: #3d3121;
  --code-bg: #f1ebe0;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #171512;
    --surface: #211e1a;
    --surface-alt: #1c1916;
    --banner-bg: #2a2418;
    --banner-border: #594b31;
    --border: #363028;
    --text: #ece7df;
    --muted: #b5ab9c;
    --accent: #cbb99d;
    --link: #dfd0b8;
    --code-bg: #2c2721;
  }
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--text); font: 16.5px/1.55 Georgia, 'Songti SC', 'Noto Serif SC', serif; }
main { max-width: 58rem; margin: 0 auto; padding: 2rem 1.25rem 4rem; }
h1, h2, h3, h4 { font-family: 'Iowan Old Style', Palatino, 'Songti SC', serif; line-height: 1.25; }
h1 { margin: 0.2rem 0 1rem; font-size: 1.9rem; }
h2 { margin: 0 0 0.35rem; font-size: 1.4rem; word-break: break-word; }
h3 { margin: 1.5rem 0 0.6rem; font-size: 1.12rem; border-bottom: 1px solid var(--border); padding-bottom: 0.25rem; }
h4 { margin: 1rem 0 0.4rem; font-size: 0.98rem; }
a { color: var(--link); }
code { background: var(--code-bg); padding: 0.08rem 0.32rem; border-radius: 3px; font: 0.86em/1.35 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
.kicker { margin: 0; letter-spacing: 0.05em; text-transform: uppercase; font: 0.78rem/1.2 ui-sans-serif, sans-serif; color: var(--accent); }
.banner { background: var(--banner-bg); border: 1px solid var(--banner-border); border-radius: 6px; padding: 0.9rem 1.1rem; }
.banner p { margin: 0.35rem 0; }
.quiet, .meta { color: var(--muted); font: 0.9rem/1.45 ui-sans-serif, sans-serif; }
article { margin-top: 2rem; background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 1.4rem 1.5rem; }
.cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(11.5rem, 1fr)); gap: 0.75rem; margin: 1rem 0; }
.card { background: var(--surface-alt); border: 1px solid var(--border); border-radius: 6px; padding: 0.7rem 0.85rem; }
.card-label { font: 0.75rem/1.2 ui-sans-serif, sans-serif; text-transform: uppercase; letter-spacing: 0.04em; color: var(--accent); }
.card-value { font: 600 1.02rem/1.3 ui-sans-serif, sans-serif; margin: 0.25rem 0 0.15rem; }
.card-sub { font: 0.8rem/1.35 ui-sans-serif, sans-serif; color: var(--muted); }
.jump { display: flex; flex-wrap: wrap; gap: 0.45rem; margin: 0.75rem 0 0.25rem; }
.jump a { font: 0.82rem/1.2 ui-sans-serif, sans-serif; text-decoration: none; padding: 0.28rem 0.6rem; border: 1px solid var(--border); border-radius: 999px; background: var(--surface-alt); }
.jump a:hover { border-color: var(--accent); }
dl { display: grid; grid-template-columns: minmax(11rem, 16rem) 1fr; gap: 0.35rem 1rem; margin: 0.5rem 0; }
dt { font: 0.86rem/1.4 ui-sans-serif, sans-serif; color: var(--accent); }
dd { margin: 0; font: 0.95rem/1.4 ui-sans-serif, sans-serif; }
.pills { list-style: none; padding: 0; margin: 0.4rem 0; display: flex; flex-wrap: wrap; gap: 0.45rem; }
.pills li { font: 0.85rem/1.3 ui-sans-serif, sans-serif; background: var(--surface-alt); border: 1px solid var(--border); border-radius: 4px; padding: 0.2rem 0.5rem; }
.hit { background: var(--surface-alt); border: 1px solid var(--border); border-radius: 6px; padding: 0.9rem 1rem; margin: 0.85rem 0; }
.hit-head { font: 0.96rem/1.4 ui-sans-serif, sans-serif; margin-bottom: 0.45rem; }
.hit-metrics { display: flex; flex-wrap: wrap; gap: 0.4rem; margin-bottom: 0.65rem; }
.badge { font: 0.8rem/1.3 ui-sans-serif, sans-serif; background: var(--surface); border: 1px solid var(--border); border-radius: 4px; padding: 0.16rem 0.48rem; color: var(--text); }
.compare { display: grid; grid-template-columns: 1fr 1fr; gap: 0.85rem; }
.pane { background: var(--surface); border: 1px solid var(--border); border-radius: 4px; padding: 0.6rem 0.8rem; }
.pane-label { margin: 0 0 0.35rem; font: 600 0.78rem/1.25 ui-sans-serif, sans-serif; text-transform: uppercase; letter-spacing: 0.03em; color: var(--accent); }
.hit-note { margin: 0.65rem 0 0; font-size: 0.82rem; }
blockquote { margin: 0; padding-left: 0.7rem; border-left: 3px solid var(--banner-border); }
blockquote p { margin: 0; word-break: break-word; }
.para { padding: 0.45rem 0; border-bottom: 1px dashed var(--border); }
.para:last-child { border-bottom: none; }
details { margin-top: 0.85rem; }
summary { cursor: pointer; font: 600 0.9rem/1.4 ui-sans-serif, sans-serif; color: var(--accent); }
.toc { margin: 1.4rem 0; padding: 1rem 1.2rem; background: var(--surface); border: 1px solid var(--border); border-radius: 8px; }
.toc h2 { margin-bottom: 0.65rem; font-size: 1.15rem; }
.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font: 0.86rem/1.4 ui-sans-serif, sans-serif; }
th, td { text-align: left; padding: 0.42rem 0.55rem; border-bottom: 1px solid var(--border); vertical-align: top; }
th { color: var(--accent); font-weight: 600; white-space: nowrap; }
.excerpt-cell { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 0.82rem; word-break: break-all; }
@media (max-width: 680px) {
  article { padding: 1rem; }
  dl { grid-template-columns: 1fr; gap: 0.15rem; }
  .compare { grid-template-columns: 1fr; }
}
@media print {
  body { background: #fff; color: #000; }
  article, .toc, .hit, .pane, .card { border-color: #bbb; background: #fff; break-inside: avoid; }
  .jump { display: none; }
}
";
