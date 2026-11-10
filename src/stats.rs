use std::collections::BTreeMap;

use serde::Serialize;
use unicode_normalization::UnicodeNormalization;

use crate::method::{
    round4, AUTHORSHIP_LINE, LAB_PROSE_LINE, LENGTH_STABLE_SENTENCES, LENGTH_UNIT,
    LENGTH_UNIT_NOTE, MIN_DUPLICATE_SENTENCE_CHARS, NOT_PLAGIARISM_LINE, NOT_PROBABILITY_LINE,
    NO_OVERALL_SCORE, REPETITION_NOTE, TRANSITION_NOTE, VOCAB_NOTE, VOCAB_STABLE_TOKENS,
};
use crate::tokenize::{
    content_hash, passage_drafts, sentences, strip_bom, tokenize, PassageDraft, Token, TokenKind,
};

#[derive(Debug, Clone, Serialize)]
pub struct Evidence {
    pub level: String,
    pub authorship: &'static str,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct TokenCounts {
    pub word: usize,
    pub cjk_or_kana: usize,
    pub numeric: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LengthSummary {
    pub unit: &'static str,
    pub count: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub sample_stdev: Option<f64>,
    pub coefficient_of_variation: Option<f64>,
    pub stable: bool,
    pub stability_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhraseCount {
    pub phrase: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepetitionSummary {
    pub duplicate_sentence_types: usize,
    pub extra_duplicate_sentences: usize,
    pub examples: Vec<String>,
    pub repeated_phrases: Vec<PhraseCount>,
    pub note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct VocabularySummary {
    pub non_numeric_tokens: usize,
    pub unique_non_numeric_tokens: usize,
    pub type_token_ratio: Option<f64>,
    pub hapax_count: usize,
    pub hapax_ratio: Option<f64>,
    pub numeric_tokens_excluded: usize,
    pub stable: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransitionSummary {
    pub inventory: Vec<PhraseCount>,
    pub total: usize,
    pub per_100_sentences: Option<f64>,
    pub note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParagraphStats {
    pub index: usize,
    pub characters: usize,
    pub sentence_count: usize,
    pub token_counts: TokenCounts,
    pub sentence_length: LengthSummary,
    pub type_token_ratio: Option<f64>,
    pub transition_total: usize,
    pub extra_duplicate_sentences: usize,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentStats {
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub token_counts: TokenCounts,
    pub sentence_length: LengthSummary,
    pub sentence_length_unit_note: &'static str,
    pub repetition: RepetitionSummary,
    pub vocabulary: VocabularySummary,
    pub transitions: TransitionSummary,
}

pub(crate) struct Analysis {
    pub bom_stripped: bool,
    pub characters: usize,
    pub evidence: Evidence,
    pub notes: Vec<String>,
    pub document: DocumentStats,
    pub paragraphs: Vec<ParagraphStats>,
    pub passages: Vec<PassageDraft>,
    pub content_hash: String,
}

pub(crate) fn analyze(text: &str) -> Analysis {
    let (bom_stripped, body) = strip_bom(text);
    let characters = body.chars().count();
    let passages = passage_drafts(text);
    let mut paragraph_stats = Vec::with_capacity(passages.len());
    let mut all_tokens: Vec<Token> = Vec::new();
    let mut all_sentences: Vec<String> = Vec::new();
    let mut all_lengths: Vec<usize> = Vec::new();

    for passage in &passages {
        let nfkc: String = passage.text.nfkc().collect();
        let sentence_texts = sentences(&nfkc);
        let sentence_token_lists: Vec<Vec<Token>> = sentence_texts
            .iter()
            .map(|sentence| tokenize(sentence))
            .collect();
        let lengths: Vec<usize> = sentence_token_lists.iter().map(Vec::len).collect();
        let sentence_tokens: Vec<Token> = sentence_token_lists.into_iter().flatten().collect();
        let transitions = transition_inventory(&sentence_tokens);
        let transition_total: usize = transitions.iter().map(|item| item.count).sum();
        let vocab = vocabulary(&passage.tokens);
        let duplicates = duplicate_sentences(&sentence_texts);
        paragraph_stats.push(ParagraphStats {
            index: passage.index,
            characters: passage.text.chars().count(),
            sentence_count: sentence_texts.len(),
            token_counts: count_kinds(&passage.tokens),
            sentence_length: length_summary(&lengths, "Paragraph sentence length"),
            type_token_ratio: vocab.type_token_ratio,
            transition_total,
            extra_duplicate_sentences: duplicates.extra_duplicate_sentences,
            preview: preview(&passage.text, 160),
        });
        all_lengths.extend(lengths);
        all_sentences.extend(sentence_texts);
        all_tokens.extend(passage.tokens.iter().cloned());
    }

    let token_counts = count_kinds(&all_tokens);
    let sentence_length = length_summary(&all_lengths, "Document sentence length");
    let non_numeric = token_counts.word + token_counts.cjk_or_kana;
    let document = DocumentStats {
        paragraph_count: passages.len(),
        sentence_count: all_sentences.len(),
        token_counts,
        sentence_length: sentence_length.clone(),
        sentence_length_unit_note: LENGTH_UNIT_NOTE,
        repetition: repetition(&all_sentences, &all_tokens),
        vocabulary: vocabulary(&all_tokens),
        transitions: transition_summary(&all_tokens, all_sentences.len()),
    };
    let evidence = evidence(characters, document.sentence_count, non_numeric);
    let notes = standing_notes(&document.token_counts, &sentence_length);
    Analysis {
        bom_stripped,
        characters,
        evidence,
        notes,
        document,
        paragraphs: paragraph_stats,
        passages,
        content_hash: content_hash(text),
    }
}

fn evidence(characters: usize, sentences: usize, non_numeric: usize) -> Evidence {
    let mut reasons = Vec::new();
    let no_words = characters == 0 || (sentences == 0 && non_numeric == 0);
    let level = if no_words {
        reasons.push("There is no readable wording to describe.".to_string());
        "insufficient"
    } else if sentences < 2 || non_numeric < 20 || characters < 80 {
        reasons.push(
            "The text is short, so length, variation, and vocabulary figures are not stable."
                .to_string(),
        );
        "insufficient"
    } else if sentences < LENGTH_STABLE_SENTENCES
        || non_numeric < VOCAB_STABLE_TOKENS
        || characters < 400
    {
        reasons.push(
            "The text supports a limited description only. A few extra sentences would move the variation and vocabulary figures."
                .to_string(),
        );
        "limited"
    } else {
        reasons.push(
            "The text is long enough for the variation and vocabulary figures to be marked stable. That still does not identify an author or a writing process."
                .to_string(),
        );
        "descriptive_only"
    };
    if !no_words && sentences < LENGTH_STABLE_SENTENCES {
        reasons.push(format!(
            "Sentence-length variation uses {sentences} sentence(s) and is marked unstable below {LENGTH_STABLE_SENTENCES}."
        ));
    }
    if !no_words && non_numeric < VOCAB_STABLE_TOKENS {
        reasons.push(format!(
            "The vocabulary ratio uses {non_numeric} non-numeric tokens and is marked unstable below {VOCAB_STABLE_TOKENS}."
        ));
    }
    reasons.push(AUTHORSHIP_LINE.to_string());
    Evidence {
        level: level.to_string(),
        authorship: AUTHORSHIP_LINE,
        reasons,
    }
}

fn standing_notes(tokens: &TokenCounts, length: &LengthSummary) -> Vec<String> {
    let mut notes = vec![
        NOT_PROBABILITY_LINE.to_string(),
        NO_OVERALL_SCORE.to_string(),
        NOT_PLAGIARISM_LINE.to_string(),
        LAB_PROSE_LINE.to_string(),
    ];
    let content = tokens.word + tokens.cjk_or_kana + tokens.numeric;
    if content > 0 && tokens.numeric * 4 >= content {
        notes.push(
            "At least a quarter of the tokens are numeric. Data tables and lab measurements often look like this.".to_string(),
        );
    }
    if length.count >= 3 {
        if let Some(median) = length.median {
            if median <= 8.0 {
                notes.push(
                    "The median sentence is very short. Procedural lab lines are often short, and this tool does not score that pattern.".to_string(),
                );
            }
        }
    }
    notes
}

fn count_kinds(tokens: &[Token]) -> TokenCounts {
    let mut counts = TokenCounts::default();
    for token in tokens {
        match token.kind {
            TokenKind::Word => counts.word += 1,
            TokenKind::CjkOrKana => counts.cjk_or_kana += 1,
            TokenKind::Numeric => counts.numeric += 1,
        }
    }
    counts
}

fn length_summary(values: &[usize], label: &str) -> LengthSummary {
    if values.is_empty() {
        return LengthSummary {
            unit: LENGTH_UNIT,
            count: 0,
            min: None,
            max: None,
            mean: None,
            median: None,
            sample_stdev: None,
            coefficient_of_variation: None,
            stable: false,
            stability_note: format!(
                "{label} uses 0 values. Below {LENGTH_STABLE_SENTENCES}, min, mean, and variation are easy to swing and are not stable."
            ),
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let count = sorted.len();
    let min = sorted[0] as f64;
    let max = sorted[count - 1] as f64;
    let sum = sorted.iter().sum::<usize>() as f64;
    let mean = sum / count as f64;
    let median = if count % 2 == 1 {
        sorted[count / 2] as f64
    } else {
        (sorted[count / 2 - 1] as f64 + sorted[count / 2] as f64) / 2.0
    };
    let sample_stdev = if count < 2 {
        None
    } else {
        let variance = sorted
            .iter()
            .map(|value| {
                let delta = *value as f64 - mean;
                delta * delta
            })
            .sum::<f64>()
            / (count as f64 - 1.0);
        Some(variance.sqrt())
    };
    let coefficient_of_variation =
        sample_stdev.and_then(|stdev| if mean > 0.0 { Some(stdev / mean) } else { None });
    let stable = count >= LENGTH_STABLE_SENTENCES;
    let stability_note = if stable {
        format!("{label} uses {count} values, which meets the reporting minimum of {LENGTH_STABLE_SENTENCES}.")
    } else {
        format!("{label} uses {count} values. Below {LENGTH_STABLE_SENTENCES}, min, mean, and variation are easy to swing and are not stable.")
    };
    LengthSummary {
        unit: LENGTH_UNIT,
        count,
        min: Some(round4(min)),
        max: Some(round4(max)),
        mean: Some(round4(mean)),
        median: Some(round4(median)),
        sample_stdev: sample_stdev.map(round4),
        coefficient_of_variation: coefficient_of_variation.map(round4),
        stable,
        stability_note,
    }
}

fn vocabulary(tokens: &[Token]) -> VocabularySummary {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut numeric = 0usize;
    for token in tokens {
        if token.kind == TokenKind::Numeric {
            numeric += 1;
            continue;
        }
        *counts.entry(token.text.clone()).or_insert(0) += 1;
    }
    let total = counts.values().sum::<usize>();
    let unique = counts.len();
    let hapax = counts.values().filter(|count| **count == 1).count();
    let stable = total >= VOCAB_STABLE_TOKENS;
    let note = if stable {
        format!(
            "The vocabulary ratio uses {total} non-numeric tokens, which meets the reporting minimum of {VOCAB_STABLE_TOKENS}. {VOCAB_NOTE}"
        )
    } else {
        format!(
            "The vocabulary ratio uses {total} non-numeric tokens and is marked unstable below {VOCAB_STABLE_TOKENS}. {VOCAB_NOTE}"
        )
    };
    VocabularySummary {
        non_numeric_tokens: total,
        unique_non_numeric_tokens: unique,
        type_token_ratio: ratio(unique, total),
        hapax_count: hapax,
        hapax_ratio: ratio(hapax, total),
        numeric_tokens_excluded: numeric,
        stable,
        note,
    }
}

fn ratio(numer: usize, denom: usize) -> Option<f64> {
    if denom == 0 {
        None
    } else {
        Some(round4(numer as f64 / denom as f64))
    }
}

fn repetition(sentences_text: &[String], tokens: &[Token]) -> RepetitionSummary {
    let duplicates = duplicate_sentences(sentences_text);
    let mut trigrams: BTreeMap<String, usize> = BTreeMap::new();
    let stream: Vec<&str> = tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Numeric)
        .map(|token| token.text.as_str())
        .collect();
    if stream.len() >= 3 {
        for window in stream.windows(3) {
            if window.iter().all(|token| is_stopword(token)) {
                continue;
            }
            let phrase = format!("{} {} {}", window[0], window[1], window[2]);
            *trigrams.entry(phrase).or_insert(0) += 1;
        }
    }
    let mut repeated: Vec<PhraseCount> = trigrams
        .into_iter()
        .filter(|(_, count)| *count >= 3)
        .map(|(phrase, count)| PhraseCount { phrase, count })
        .collect();
    repeated.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.phrase.cmp(&right.phrase))
    });
    repeated.truncate(8);
    RepetitionSummary {
        duplicate_sentence_types: duplicates.duplicate_sentence_types,
        extra_duplicate_sentences: duplicates.extra_duplicate_sentences,
        examples: duplicates.examples,
        repeated_phrases: repeated,
        note: REPETITION_NOTE,
    }
}

struct DuplicateSentences {
    duplicate_sentence_types: usize,
    extra_duplicate_sentences: usize,
    examples: Vec<String>,
}

fn duplicate_sentences(sentences_text: &[String]) -> DuplicateSentences {
    let mut counts: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for sentence in sentences_text {
        let normalized = crate::tokenize::normalize_exact(sentence);
        if normalized.chars().count() < MIN_DUPLICATE_SENTENCE_CHARS {
            continue;
        }
        let entry = counts
            .entry(normalized)
            .or_insert_with(|| (0, sentence.clone()));
        entry.0 += 1;
    }
    let mut duplicates: Vec<(usize, String)> = counts
        .into_values()
        .filter(|(count, _)| *count >= 2)
        .collect();
    duplicates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    let extra = duplicates.iter().map(|(count, _)| count - 1).sum();
    let examples = duplicates
        .iter()
        .take(5)
        .map(|(_, text)| preview(text, 120))
        .collect();
    DuplicateSentences {
        duplicate_sentence_types: duplicates.len(),
        extra_duplicate_sentences: extra,
        examples,
    }
}

fn transition_summary(tokens: &[Token], sentence_count: usize) -> TransitionSummary {
    let inventory = transition_inventory(tokens);
    let total = inventory.iter().map(|item| item.count).sum();
    let per_100_sentences = if sentence_count == 0 {
        None
    } else {
        Some(round4(total as f64 * 100.0 / sentence_count as f64))
    };
    TransitionSummary {
        inventory,
        total,
        per_100_sentences,
        note: TRANSITION_NOTE,
    }
}

fn transition_inventory(tokens: &[Token]) -> Vec<PhraseCount> {
    let mut found: Vec<PhraseCount> = Vec::new();
    let phrases = english_phrases();
    let mut english_counts = vec![0usize; phrases.len()];
    for segment in word_segments(tokens) {
        let mut index = 0;
        while index < segment.len() {
            let mut matched = false;
            for (phrase_index, phrase) in phrases.iter().enumerate() {
                if index + phrase.len() <= segment.len()
                    && segment[index..index + phrase.len()]
                        .iter()
                        .zip(phrase.iter())
                        .all(|(token, expected)| token == expected)
                {
                    english_counts[phrase_index] += 1;
                    index += phrase.len();
                    matched = true;
                    break;
                }
            }
            if !matched {
                index += 1;
            }
        }
    }
    for (phrase, count) in phrases.iter().zip(english_counts) {
        if count > 0 {
            found.push(PhraseCount {
                phrase: phrase.join(" "),
                count,
            });
        }
    }

    let cjk: String = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::CjkOrKana)
        .map(|token| token.text.as_str())
        .collect();
    let cjk_chars: Vec<char> = cjk.chars().collect();
    let chinese = chinese_phrases();
    let mut chinese_counts = vec![0usize; chinese.len()];
    let mut index = 0;
    while index < cjk_chars.len() {
        let mut matched = false;
        for (phrase_index, phrase) in chinese.iter().enumerate() {
            let phrase_chars: Vec<char> = phrase.chars().collect();
            if cjk_chars[index..].starts_with(phrase_chars.as_slice()) {
                chinese_counts[phrase_index] += 1;
                index += phrase_chars.len();
                matched = true;
                break;
            }
        }
        if !matched {
            index += 1;
        }
    }
    for (phrase, count) in chinese.iter().zip(chinese_counts) {
        if count > 0 {
            found.push(PhraseCount {
                phrase: (*phrase).to_string(),
                count,
            });
        }
    }
    found
}

fn word_segments(tokens: &[Token]) -> Vec<Vec<String>> {
    let mut segments = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        if token.kind == TokenKind::Word {
            current.push(token.text.clone());
        } else if !current.is_empty() {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

fn english_phrases() -> Vec<Vec<&'static str>> {
    let mut phrases = vec![
        vec!["on", "the", "other", "hand"],
        vec!["in", "particular"],
        vec!["in", "addition"],
        vec!["in", "contrast"],
        vec!["in", "conclusion"],
        vec!["in", "summary"],
        vec!["for", "example"],
        vec!["for", "instance"],
        vec!["as", "a", "result"],
        vec!["however"],
        vec!["therefore"],
        vec!["moreover"],
        vec!["furthermore"],
        vec!["consequently"],
        vec!["nevertheless"],
        vec!["additionally"],
        vec!["meanwhile"],
        vec!["overall"],
        vec!["finally"],
    ];
    phrases.sort_by_key(|phrase| std::cmp::Reverse(phrase.len()));
    phrases
}

fn chinese_phrases() -> Vec<&'static str> {
    let mut phrases = vec![
        "另一方面",
        "与此同时",
        "也就是说",
        "由此可见",
        "换言之",
        "总的来说",
        "除此之外",
        "然而",
        "因此",
        "此外",
        "另外",
        "总之",
        "综上",
        "例如",
        "所以",
        "但是",
        "同时",
        "最后",
    ];
    phrases.sort_by_key(|phrase| std::cmp::Reverse(phrase.chars().count()));
    phrases
}

fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "the"
            | "of"
            | "and"
            | "or"
            | "to"
            | "in"
            | "for"
            | "on"
            | "with"
            | "from"
            | "by"
            | "as"
            | "at"
            | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "it"
            | "this"
            | "that"
            | "these"
            | "those"
            | "we"
            | "our"
            | "you"
            | "they"
            | "their"
            | "into"
            | "over"
            | "after"
            | "before"
            | "not"
            | "if"
            | "but"
    )
}

pub(crate) fn preview(text: &str, max_chars: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let count = collapsed.chars().count();
    if count <= max_chars {
        collapsed
    } else {
        let trimmed: String = collapsed.chars().take(max_chars).collect();
        format!("{trimmed}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_length_mean_and_sample_stdev() {
        let analysis = analyze("One two three four. One two. One two three four five six.");
        assert_eq!(analysis.document.sentence_count, 3);
        let length = &analysis.document.sentence_length;
        assert_eq!(length.min, Some(2.0));
        assert_eq!(length.max, Some(6.0));
        assert_eq!(length.mean, Some(4.0));
        assert_eq!(length.median, Some(4.0));
        assert_eq!(length.sample_stdev, Some(2.0));
        assert_eq!(length.coefficient_of_variation, Some(0.5));
        assert!(!length.stable);
        assert_eq!(analysis.evidence.level, "insufficient");
        assert_eq!(analysis.evidence.authorship, AUTHORSHIP_LINE);
    }

    #[test]
    fn empty_short_and_transitions() {
        let empty = analyze("");
        assert_eq!(empty.evidence.level, "insufficient");
        assert!(empty.document.sentence_length.mean.is_none());
        assert!(empty.notes.iter().any(|note| note == NO_OVERALL_SCORE));

        let short = analyze("   \n\n  ");
        assert_eq!(short.evidence.level, "insufficient");

        let transitions =
            analyze("However, the sample cooled slowly today. Therefore we stopped the trial.");
        let phrases: Vec<&str> = transitions
            .document
            .transitions
            .inventory
            .iter()
            .map(|item| item.phrase.as_str())
            .collect();
        assert!(phrases.contains(&"however"));
        assert!(phrases.contains(&"therefore"));

        let chinese = analyze("因此温度下降。但是我们停止观察。");
        let chinese_phrases: Vec<&str> = chinese
            .document
            .transitions
            .inventory
            .iter()
            .map(|item| item.phrase.as_str())
            .collect();
        assert!(chinese_phrases.contains(&"因此"));
        assert!(chinese_phrases.contains(&"但是"));
        assert!(chinese.document.token_counts.cjk_or_kana > 0);
    }

    #[test]
    fn repeated_sentences_are_counted_without_a_verdict() {
        let analysis = analyze(
            "We measured the glass beaker again. We measured the glass beaker again. We measured the glass beaker again.",
        );
        assert_eq!(analysis.document.repetition.duplicate_sentence_types, 1);
        assert_eq!(analysis.document.repetition.extra_duplicate_sentences, 2);
        assert!(analysis.document.repetition.note.contains("not a verdict"));
    }

    #[test]
    fn numeric_share_adds_a_lab_note_not_a_score() {
        let analysis =
            analyze("Run 1 used 10 20 30 40. Run 2 used 11 21 31 41. Run 3 used 12 22 32 42.");
        assert!(analysis
            .notes
            .iter()
            .any(|note| note.contains("tokens are numeric")));
        assert!(analysis
            .notes
            .iter()
            .any(|note| note.contains("not an AI probability")));
    }
}
