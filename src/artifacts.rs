use serde::Serialize;

use crate::method::{ARTIFACT_DESC, ARTIFACT_EXTRACTED};

const MAX_FINDINGS: usize = 24;

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactReport {
    pub scope: String,
    pub newline_style: String,
    pub findings: Vec<ArtifactFinding>,
    pub truncated_count: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactFinding {
    pub kind: String,
    pub code_point: String,
    pub line: usize,
    pub column: usize,
    pub char_index: usize,
    pub excerpt: String,
}

pub(crate) fn audit_text(text: &str, origin: &str) -> ArtifactReport {
    let extracted = origin == "docx" || origin == "pdf";
    let chars: Vec<char> = text.chars().collect();
    let mut findings = Vec::new();
    let mut omitted = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;
    let mut index = 0usize;
    let mut saw_lf = false;
    let mut saw_crlf = false;
    let mut saw_cr = false;
    let mut crlf_listed = 0usize;
    let mut cr_listed = 0usize;
    let mut trailing_listed = 0usize;
    let mut ideographic_spaces = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '\r' {
            let paired = index + 1 < chars.len() && chars[index + 1] == '\n';
            if !extracted {
                if let Some(start) = trailing_ascii_whitespace(&chars, index) {
                    if trailing_listed < 8 {
                        push(
                            &mut findings,
                            &mut omitted,
                            finding(
                                "trailing_whitespace",
                                "U+0020",
                                line,
                                column_at(&chars, start),
                                start,
                                &chars,
                                start..index,
                            ),
                        );
                        trailing_listed += 1;
                    } else {
                        omitted += 1;
                    }
                }
                if paired {
                    saw_crlf = true;
                    if crlf_listed < 8 {
                        push(
                            &mut findings,
                            &mut omitted,
                            finding(
                                "crlf",
                                "CRLF",
                                line,
                                column,
                                index,
                                &chars,
                                index..index + 2,
                            ),
                        );
                        crlf_listed += 1;
                    } else {
                        omitted += 1;
                    }
                } else {
                    saw_cr = true;
                    if cr_listed < 8 {
                        push(
                            &mut findings,
                            &mut omitted,
                            finding(
                                "bare_cr",
                                "U+000D",
                                line,
                                column,
                                index,
                                &chars,
                                index..index + 1,
                            ),
                        );
                        cr_listed += 1;
                    } else {
                        omitted += 1;
                    }
                }
            }
            if paired {
                index += 2;
            } else {
                index += 1;
            }
            line += 1;
            column = 1;
            continue;
        }
        if ch == '\n' {
            if !extracted {
                saw_lf = true;
                if let Some(start) = trailing_ascii_whitespace(&chars, index) {
                    if trailing_listed < 8 {
                        push(
                            &mut findings,
                            &mut omitted,
                            finding(
                                "trailing_whitespace",
                                "U+0020",
                                line,
                                column_at(&chars, start),
                                start,
                                &chars,
                                start..index,
                            ),
                        );
                        trailing_listed += 1;
                    } else {
                        omitted += 1;
                    }
                }
            }
            index += 1;
            line += 1;
            column = 1;
            continue;
        }
        if ch == '\u{3000}' {
            ideographic_spaces += 1;
        } else if let Some(kind) = hidden_kind(ch) {
            push(
                &mut findings,
                &mut omitted,
                finding(
                    kind,
                    &format!("U+{code:04X}", code = ch as u32),
                    line,
                    column,
                    index,
                    &chars,
                    index..index + 1,
                ),
            );
        }
        index += 1;
        column += 1;
    }

    let newline_style = if extracted {
        "not_reported".to_string()
    } else if (saw_lf && (saw_crlf || saw_cr)) || (saw_crlf && saw_cr) {
        "mixed".to_string()
    } else if saw_crlf {
        "crlf".to_string()
    } else if saw_cr {
        "cr".to_string()
    } else if saw_lf {
        "lf".to_string()
    } else {
        "none".to_string()
    };

    let scope = if extracted {
        format!("extracted {origin} text")
    } else {
        "original UTF-8 file".to_string()
    };
    let mut notes = vec![ARTIFACT_DESC.to_string()];
    if extracted {
        notes.push(ARTIFACT_EXTRACTED.to_string());
    } else {
        notes.push(format!(
            "Newline style is {newline_style}. CRLF and bare CR are listed up to 8 times each; further copies are counted in truncated_count."
        ));
    }
    if ideographic_spaces > 0 {
        notes.push(format!(
            "{ideographic_spaces} ideographic space(s) (U+3000). These are ordinary in Chinese typesetting and are not listed as hidden controls."
        ));
    }
    if findings.is_empty() {
        notes.push(
            "No hidden controls or unusual spaces were found. For a UTF-8 file, unusual newlines would be listed here too."
                .to_string(),
        );
    }
    ArtifactReport {
        scope,
        newline_style,
        findings,
        truncated_count: omitted,
        notes,
    }
}

pub(crate) fn show_text(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if ch == '\n' || ch == '\t' {
            out.push(ch);
        } else if hidden_kind(ch).is_some() || ch == '\r' || ch.is_control() {
            out.push_str(&format!("U+{code:04X}", code = ch as u32));
        } else {
            out.push(ch);
        }
    }
    out
}

fn hidden_kind(ch: char) -> Option<&'static str> {
    match ch {
        '\u{FEFF}' => Some("bom"),
        '\u{00A0}' | '\u{202F}' | '\u{2007}' => Some("nbsp"),
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{180E}' => Some("zero_width"),
        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => {
            Some("bidi_control")
        }
        '\u{00AD}' => Some("soft_hyphen"),
        '\u{206A}'..='\u{206F}' | '\u{034F}' | '\u{FFF9}'..='\u{FFFB}' => Some("other_format"),
        _ => None,
    }
}

fn trailing_ascii_whitespace(chars: &[char], newline_index: usize) -> Option<usize> {
    if newline_index == 0 {
        return None;
    }
    let mut cursor = newline_index;
    while cursor > 0 && matches!(chars[cursor - 1], ' ' | '\t') {
        cursor -= 1;
    }
    if cursor == newline_index {
        None
    } else {
        Some(cursor)
    }
}

fn column_at(chars: &[char], index: usize) -> usize {
    let mut column = 1usize;
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        if chars[cursor] == '\n' || chars[cursor] == '\r' {
            break;
        }
        column += 1;
    }
    column
}

fn finding(
    kind: &str,
    code_point: &str,
    line: usize,
    column: usize,
    char_index: usize,
    chars: &[char],
    span: std::ops::Range<usize>,
) -> ArtifactFinding {
    let start = span.start.saturating_sub(12);
    let end = (span.end + 12).min(chars.len());
    let excerpt: String = chars[start..end].iter().collect();
    ArtifactFinding {
        kind: kind.to_string(),
        code_point: code_point.to_string(),
        line,
        column,
        char_index,
        excerpt: show_text(&excerpt),
    }
}

fn push(findings: &mut Vec<ArtifactFinding>, omitted: &mut usize, finding: ArtifactFinding) {
    if findings.len() < MAX_FINDINGS {
        findings.push(finding);
    } else {
        *omitted += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_nbsp_bidi_and_zero_width_have_locations() {
        let text =
            "Line one.\r\nTrailing \r\nNBSP\u{00A0}here\u{200B} and\u{202E}bidi.\n结果正常。\n";
        let report = audit_text(text, "utf-8");
        assert_eq!(report.newline_style, "mixed");
        assert!(report
            .findings
            .iter()
            .any(|item| { item.kind == "crlf" && item.line == 1 && item.code_point == "CRLF" }));
        assert!(report
            .findings
            .iter()
            .any(|item| item.kind == "trailing_whitespace" && item.line == 2));
        assert!(report
            .findings
            .iter()
            .any(|item| item.kind == "nbsp" && item.code_point == "U+00A0"));
        assert!(report
            .findings
            .iter()
            .any(|item| item.kind == "zero_width" && item.code_point == "U+200B"));
        let bidi = report
            .findings
            .iter()
            .find(|item| item.kind == "bidi_control")
            .unwrap();
        assert_eq!(bidi.code_point, "U+202E");
        assert!(!bidi.excerpt.contains('\u{202E}'));
        assert!(bidi.excerpt.contains("U+202E"));
        assert!(report
            .notes
            .iter()
            .any(|note| note.contains("not an AI-generation rate")));
        assert!(report.notes.iter().any(|note| note.contains("tokenizer")));
    }

    #[test]
    fn ordinary_chinese_is_not_flagged_and_extracted_text_skips_newlines() {
        let clean = audit_text("温度下降较快，随后变慢。我们没有测量室温。\n", "utf-8");
        assert!(clean.findings.is_empty(), "{:?}", clean.findings);
        assert_eq!(clean.newline_style, "lf");

        let extracted = audit_text("Line\r\nwith\u{202E}mark\n", "pdf");
        assert_eq!(extracted.newline_style, "not_reported");
        assert!(extracted.findings.iter().all(|item| item.kind != "crlf"));
        assert!(extracted
            .findings
            .iter()
            .any(|item| item.kind == "bidi_control"));
        assert!(extracted
            .notes
            .iter()
            .any(|note| note.contains("extractor inserts paragraph breaks")));
    }
}
