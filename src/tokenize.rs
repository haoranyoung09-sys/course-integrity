use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::method::{MIN_NORMALIZED_CHARS, MIN_SHARED_SHINGLES, SHINGLE_K};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    CjkOrKana,
    Numeric,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShingleScore {
    pub shared: usize,
    pub query_shingles: usize,
    pub source_shingles: usize,
    pub containment_basis_points: Option<u32>,
    pub jaccard_basis_points: Option<u32>,
    pub containment: Option<f64>,
    pub jaccard: Option<f64>,
}

#[derive(Debug, Clone)]
pub(crate) struct PassageDraft {
    pub index: usize,
    pub text: String,
    pub normalized: String,
    pub norm_hash: String,
    pub tokens: Vec<Token>,
    pub shingles: BTreeSet<String>,
    pub comparable: bool,
    pub exact_eligible: bool,
}

pub fn strip_bom(text: &str) -> (bool, &str) {
    match text.strip_prefix('\u{feff}') {
        Some(rest) => (true, rest),
        None => (false, text),
    }
}

pub fn normalize_exact(input: &str) -> String {
    let folded: String = input.nfkc().collect::<String>().to_lowercase();
    let mut out = String::new();
    let mut pending_space = false;
    for ch in folded.chars() {
        if ch.is_whitespace() || is_separator(ch) {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(ch);
    }
    out
}

pub fn content_hash(text: &str) -> String {
    sha256_hex(&normalize_exact(text))
}

pub fn paragraphs(text: &str) -> Vec<String> {
    let (_, body) = strip_bom(text);
    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    let mut paras = Vec::new();
    let mut buf: Vec<&str> = Vec::new();
    for line in normalized.split('\n') {
        if line.trim().is_empty() {
            if !buf.is_empty() {
                paras.push(buf.join("\n"));
                buf.clear();
            }
        } else {
            buf.push(line);
        }
    }
    if !buf.is_empty() {
        paras.push(buf.join("\n"));
    }
    paras
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let folded: String = input.nfkc().collect::<String>().to_lowercase();
    let chars: Vec<char> = folded.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() || is_separator(ch) {
            i += 1;
            continue;
        }
        if is_cjk_or_kana(ch) {
            tokens.push(Token {
                text: ch.to_string(),
                kind: TokenKind::CjkOrKana,
            });
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i].is_ascii_digit() {
                    i += 1;
                    continue;
                }
                if (chars[i] == '.' || chars[i] == ',')
                    && i + 1 < chars.len()
                    && chars[i + 1].is_ascii_digit()
                {
                    i += 1;
                    continue;
                }
                break;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::Numeric,
            });
            continue;
        }
        if is_word_char(ch) {
            let start = i;
            i += 1;
            while i < chars.len() {
                if is_word_char(chars[i]) {
                    i += 1;
                    continue;
                }
                if matches!(chars[i], '\'' | '\u{2019}')
                    && i + 1 < chars.len()
                    && is_word_char(chars[i + 1])
                {
                    i += 1;
                    continue;
                }
                break;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::Word,
            });
            continue;
        }
        i += 1;
    }
    tokens
}

pub fn shingles(tokens: &[Token]) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    if tokens.len() < SHINGLE_K {
        return set;
    }
    for window in tokens.windows(SHINGLE_K) {
        let phrase = window
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        set.insert(phrase);
    }
    set
}

pub fn score_shingles(query: &BTreeSet<String>, source: &BTreeSet<String>) -> ShingleScore {
    let shared = query.intersection(source).count();
    let union = query.len() + source.len() - shared;
    let (containment_basis_points, containment) = ratio(shared, query.len());
    let (jaccard_basis_points, jaccard) = ratio(shared, union);
    ShingleScore {
        shared,
        query_shingles: query.len(),
        source_shingles: source.len(),
        containment_basis_points,
        jaccard_basis_points,
        containment,
        jaccard,
    }
}

fn ratio(numer: usize, denom: usize) -> (Option<u32>, Option<f64>) {
    if denom == 0 {
        return (None, None);
    }
    let points = ((numer as u128) * 10_000 + (denom as u128 / 2)) / (denom as u128);
    let points = u32::try_from(points).unwrap_or(10_000);
    let rendered = format!("{}.{:04}", points / 10_000, points % 10_000);
    let value = rendered.parse::<f64>().unwrap_or(0.0);
    (Some(points), Some(value))
}

pub fn sentences(nfkc_text: &str) -> Vec<String> {
    let chars: Vec<char> = nfkc_text.chars().collect();
    let mut buf = String::new();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\n' {
            push_sentence(&mut out, &mut buf);
            i += 1;
            continue;
        }
        buf.push(ch);
        if is_sentence_ender(ch)
            && !is_decimal_dot(&chars, i)
            && !is_multidot_abbreviation(&chars, i)
            && !is_abbreviation(&buf)
        {
            i += 1;
            while i < chars.len() && is_closer(chars[i]) {
                buf.push(chars[i]);
                i += 1;
            }
            push_sentence(&mut out, &mut buf);
            continue;
        }
        i += 1;
    }
    push_sentence(&mut out, &mut buf);
    out.into_iter()
        .map(|sentence| sentence.trim().to_string())
        .filter(|sentence| !sentence.is_empty() && !tokenize(sentence).is_empty())
        .collect()
}

pub(crate) fn passage_drafts(text: &str) -> Vec<PassageDraft> {
    paragraphs(text)
        .into_iter()
        .enumerate()
        .map(|(offset, paragraph)| {
            let normalized = normalize_exact(&paragraph);
            let nfkc: String = paragraph.nfkc().collect();
            let tokens = tokenize(&nfkc);
            let shingles = shingles(&tokens);
            let exact_eligible = normalized.chars().count() >= MIN_NORMALIZED_CHARS;
            let comparable = exact_eligible && shingles.len() >= MIN_SHARED_SHINGLES;
            PassageDraft {
                index: offset + 1,
                norm_hash: sha256_hex(&normalized),
                text: paragraph,
                normalized,
                tokens,
                shingles,
                comparable,
                exact_eligible,
            }
        })
        .collect()
}

pub(crate) fn canonical_label(label: &str) -> String {
    let replaced = label.trim().replace('\\', "/");
    replaced.strip_prefix("./").unwrap_or(&replaced).to_string()
}

fn push_sentence(out: &mut Vec<String>, buf: &mut String) {
    if !buf.trim().is_empty() {
        out.push(std::mem::take(buf));
    } else {
        buf.clear();
    }
}

fn is_sentence_ender(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | ';' | '。' | '！' | '？' | '；' | '…')
}

fn is_closer(ch: char) -> bool {
    matches!(
        ch,
        '"' | '\'' | '\u{201d}' | '\u{2019}' | '』' | '」' | ')' | ']' | '）'
    )
}

fn is_decimal_dot(chars: &[char], index: usize) -> bool {
    chars[index] == '.'
        && index > 0
        && index + 1 < chars.len()
        && chars[index - 1].is_ascii_digit()
        && chars[index + 1].is_ascii_digit()
}

fn is_multidot_abbreviation(chars: &[char], index: usize) -> bool {
    if chars.get(index).copied() != Some('.') || index == 0 {
        return false;
    }
    let letter = chars[index - 1].to_ascii_lowercase();
    let next = chars.get(index + 1).map(|ch| ch.to_ascii_lowercase());
    let after = chars.get(index + 2).copied();
    let boundary = index == 1 || !chars[index - 2].is_alphabetic();
    boundary && after == Some('.') && matches!((letter, next), ('e', Some('g')) | ('i', Some('e')))
}

fn is_abbreviation(buf_including_dot: &str) -> bool {
    let Some(without) = buf_including_dot.strip_suffix('.') else {
        return false;
    };
    let lower = without.trim_end().to_lowercase();
    const SUFFIXES: &[&str] = &[
        "mr", "mrs", "ms", "dr", "prof", "fig", "eq", "eqs", "etc", "e.g", "i.e", "vs", "approx",
        "no", "vol", "pp", "al", "st",
    ];
    SUFFIXES.iter().any(|suffix| {
        if !lower.ends_with(suffix) {
            return false;
        }
        let boundary = lower.len() - suffix.len();
        boundary == 0
            || !lower[..boundary]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_alphabetic())
    })
}

fn is_cjk_or_kana(ch: char) -> bool {
    matches!(
        ch,
        '\u{3040}'..='\u{30FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{20000}'..='\u{2A6DF}'
    )
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphabetic() && !is_cjk_or_kana(ch)
}

fn is_separator(ch: char) -> bool {
    !ch.is_whitespace() && !ch.is_alphanumeric() && !is_cjk_or_kana(ch)
}

fn sha256_hex(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_whitespace_have_no_tokens() {
        assert!(tokenize("").is_empty());
        assert!(tokenize(" \n\t ").is_empty());
        assert!(paragraphs("\n\n").is_empty());
        assert!(sentences("").is_empty());
    }

    #[test]
    fn fullwidth_latin_and_cjk_and_decimal() {
        assert_eq!(tokenize("Ｈｅｌｌｏ")[0].text, "hello");
        assert_eq!(normalize_exact("Ｈｅｌｌｏ"), normalize_exact("Hello"));
        assert_eq!(tokenize("naïve")[0].text, "naïve");
        let cjk = tokenize("结果");
        assert_eq!(
            cjk.iter()
                .map(|token| token.text.as_str())
                .collect::<Vec<_>>(),
            vec!["结", "果"]
        );
        let mixed = tokenize("21.5 °C and H2O");
        assert!(mixed.iter().any(|token| token.text == "21.5"));
        assert!(mixed.iter().any(|token| token.text == "c"));
        assert!(mixed.iter().any(|token| token.text == "h"));
        assert!(mixed
            .iter()
            .any(|token| token.kind == TokenKind::Numeric && token.text == "2"));
        assert_eq!(tokenize("don't")[0].text, "don't");
    }

    #[test]
    fn sentence_boundaries_handle_abbrev_decimal_and_chinese() {
        assert_eq!(
            sentences("Dr. Smith measured the temperature carefully today.").len(),
            1
        );
        assert_eq!(
            sentences("Dr. Smith left. Then we stopped completely.").len(),
            2
        );
        assert_eq!(sentences("The value was 3.14 exactly today.").len(), 1);
        assert_eq!(sentences("温度下降。然后我们停止。").len(), 2);
        assert_eq!(sentences("see e.g. the beaker today.").len(), 1);
    }

    #[test]
    fn worked_containment_example() {
        let query = tokenize("alpha bravo charlie delta echo foxtrot golf hotel");
        let source = tokenize("alpha bravo charlie delta echo foxtrot golf");
        let scored = score_shingles(&shingles(&query), &shingles(&source));
        assert_eq!(scored.query_shingles, 5);
        assert_eq!(scored.source_shingles, 4);
        assert_eq!(scored.shared, 4);
        assert_eq!(scored.containment_basis_points, Some(8000));
        assert_eq!(scored.jaccard_basis_points, Some(8000));
        assert_eq!(scored.containment, Some(0.8));
        assert_eq!(scored.jaccard, Some(0.8));
    }

    #[test]
    fn shingles_keep_numeric_tokens() {
        let tokens = tokenize("value 21 was recorded here now");
        let grams = shingles(&tokens);
        assert!(grams.iter().any(|gram| gram.contains("21")));
    }

    #[test]
    fn short_banner_cannot_exact_match() {
        assert!(normalize_exact("Synthetic fixture.").chars().count() < MIN_NORMALIZED_CHARS);
    }
}
