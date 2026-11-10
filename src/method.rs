//! Wording locked to the report, the README, and `docs/methodology.md`.

pub const PRODUCT: &str = "course-integrity";
pub const SCHEMA: &str = "course-integrity.review.v1";
pub const CORPUS_SCHEMA: &str = "course-integrity.corpus.v1";
pub const DB_SCHEMA_VERSION: &str = "1";

pub const SHINGLE_K: usize = 4;
pub const MIN_SHARED_SHINGLES: usize = 4;
pub const MIN_NORMALIZED_CHARS: usize = 40;
pub const MIN_DUPLICATE_SENTENCE_CHARS: usize = 20;
pub const DEFAULT_MIN_CONTAINMENT: f64 = 0.8;
pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_REPORTED_MATCHES: usize = 20;
pub const VOCAB_STABLE_TOKENS: usize = 80;
pub const LENGTH_STABLE_SENTENCES: usize = 5;

pub const DISCLAIMER: &str = "Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.";

pub const AUTHORSHIP_LINE: &str = "Insufficient evidence for authorship. These signals do not identify a person or a writing process.";

pub const NOT_PROBABILITY_LINE: &str = "Unvalidated surface heuristics are reported as descriptive review signals. They are not an AI probability, a calibrated risk score, a confidence interval, or an accuracy claim.";

pub const NO_OVERALL_SCORE: &str = "This report has no overall score. The counts are not weighted, summed, or converted into a risk label.";

pub const NOT_PLAGIARISM_LINE: &str = "Similarity is not a plagiarism verdict. Shared prompts, quotations, templates, group work, and ordinary technical phrasing can produce a match.";

pub const LAB_PROSE_LINE: &str = "Technical lab prose often uses short procedural sentences, repeated units, and number-heavy lines. Those patterns are described here and are not treated as an integrity finding.";

pub const NOT_QUERIED_LINE: &str = "No local corpus was queried because --db was not set. Overlap was not checked. This is not a plagiarism check and it does not contact a remote service.";

pub const SCOPE_LINE: &str = "Coverage is only this local database. It is not the web, not an institution-wide archive, and not a commercial checker corpus. This tool does not connect to Turnitin, CNKI, or any other remote service, and it does not claim to reproduce their results.";

pub const PERSISTED_TEXT_NOTE: &str = "This local SQLite database stores the full original text of each added UTF-8 file that had a matchable passage, or the extracted plain text of a DOCX or PDF, plus a normalized copy of each passage and its shingles. The original DOCX or PDF bytes are not stored. The tool does not upload that text. Do not commit the database, and do not point it at coursework you are not allowed to keep.";

pub const TOKENIZER_DESC: &str = "UTF-8 text is NFKC-normalized and lowercased with Unicode simple lowercase (not full casefold). A word token is a run of letters, including letters with diacritics, and an apostrophe stays inside a word when letters surround it. A numeric token is a run of ASCII digits with an optional internal decimal point or comma between digits. Each Han character, kana character, or CJK compatibility ideograph is its own token. Punctuation, symbols, emoji, and a leading byte-order mark are separators, not tokens. Digits split mixed tokens, so H2O becomes three tokens. Hyphens are separators. There is no Chinese dictionary word segmenter.";

pub const COUNT_STREAMS: &str = "Three counts are used. Sentence length counts every token, including numbers. Vocabulary and type-token ratio omit numeric tokens and still include function words. Shingles keep every token, including numbers and function words, so a copied measurement sentence can still match. English function words are removed only when a whole repeated trigram is made of them.";

pub const SHINGLE_DESC: &str = "A shingle is an ordered sequence of 4 consecutive tokens from one paragraph, joined by single spaces. Comparison uses the set of distinct shingles, so a repeated shingle counts once. Punctuation is not a token, so a shingle can cross a comma or a line break inside the paragraph. A passage with fewer than 4 distinct shingles, or shorter than 40 normalized characters, is not near-exact compared. The same width is used for English words and for Chinese characters.";

pub const CONTAINMENT_DENOMINATOR: &str = "Containment = |distinct query shingles intersect distinct source shingles| / |distinct query shingles|. The denominator is the number of distinct 4-token shingles in the query passage, not the source passage, not the character length, and not a probability. The ratio is rounded half-up to 4 decimal places. containment_basis_points uses the same rounding, and 10000 means 1. The threshold comparison uses those basis points.";

pub const JACCARD_DESC: &str = "Jaccard = |intersection| / |union| of those same distinct shingle sets. It is reported beside containment, with the same half-up rounding and jaccard_basis_points. The near-exact threshold uses containment only. The default threshold is 0.8 (8000 basis points), together with at least 4 shared shingles.";

pub const UNICODE_DESC: &str = "Files must be valid UTF-8. A leading U+FEFF is ignored for statistics and does not change the normalized hash. CR, LF, and CRLF are line breaks. A blank line (Unicode whitespace only) separates paragraphs. Compatibility forms are folded with NFKC before tokenization and hashing, so fullwidth Latin letters match ASCII letters. The degree Celsius sign folds under NFKC; the degree mark is a separator. Case folding is Unicode simple lowercase, so German eszett is not expanded to ss.";

pub const EXACT_DESC: &str = "An exact passage match means the normalized passage strings are equal and the normalized form has at least 40 characters. Normalization for identity is NFKC, simple lowercase, punctuation and symbols removed, and whitespace collapsed to single spaces. Shorter passages are not emitted as matches, because short technical phrases collide often.";

pub const HASH_DESC: &str = "Document identity for storage deduplication is the SHA-256 hex digest of the normalized full text, not of the raw bytes. The database still keeps the original text, including a leading BOM if one was present. Adding the same normalized document again does not insert a second copy.";

pub const SELF_DESC: &str = "Checking a file excludes stored rows with the same source label, so a file is not reported as overlapping its own stored copy. Identical normalized text under a different label is reported as a match. Pass --include-self to compare a file with rows stored under the same label. Self-exclusion is by label, not by hash: a classmate file with the same text and a different label is still a match.";

pub const LENGTH_UNIT: &str = "tokens";

pub const LENGTH_UNIT_NOTE: &str = "Sentence length counts tokens: an English word, a numeric run, or one Han or kana character. English word lengths and Chinese character lengths are not on the same scale, and this tool does not compare them. Sample standard deviation uses the n-1 divisor and is omitted below 2 sentences. Variation is marked stable only at 5 or more sentences. Reported decimals are rounded to 4 places.";

pub const VOCAB_NOTE: &str = "Type-token ratio is unique non-numeric tokens divided by all non-numeric tokens in the counted span. It falls as texts get longer. Chinese tokens are characters, so a Chinese ratio is not comparable to an English word ratio. Below 80 non-numeric tokens the ratio is marked unstable. It is not an authorship score.";

pub const REPETITION_NOTE: &str = "Repeated phrases are trigrams of the non-numeric token stream that occur at least 3 times. Numeric tokens are left out of that stream. Trigrams whose three tokens are all common English function words are omitted. At most 8 phrases are listed. Duplicate sentences are exact matches after the same normalization used for corpus identity, and only when the normalized sentence has at least 20 characters. Repetition is a description, not a verdict.";

pub const TRANSITION_NOTE: &str = "Transition phrases are a fixed inventory counted for description. English phrases match consecutive word tokens, longest phrase first, and a numeric or Han token breaks the word run. Chinese phrases match the Han and kana character stream with separators removed, longest phrase first, without overlapping. Step words such as first, second, and then are not in the inventory, because lab procedures repeat them constantly. A high or low count is not a style verdict and is not evidence about authorship.";

pub const EXCERPT_NOTE: &str = "Excerpts are the original passage text with whitespace collapsed, truncated to 400 characters. Paragraph indexes are 1-based and count blank-line paragraphs, including short ones that are not themselves matchable.";

pub const BELOW_THRESHOLD_NOTE: &str = "Shown for transparency only. This pair does not meet the exact or near-exact threshold and is not a match.";

pub const DOCUMENT_EXACT_NOTE: &str = "The normalized full text matches a stored document with a different source label. Identical normalized text is a descriptive overlap signal, not a plagiarism verdict and not proof of who copied whom.";

pub const EXTRACTION_DESC: &str = "DOCX text is read from word/document.xml and from word header and footer XML. Paragraph locations name that file and the paragraph number. PDF text is read from each page's text layer, and locations name the page. Images, drawings, charts, text boxes, and legacy .doc files are not read. A Word or PDF file is text extraction, not multimodal reasoning.";

pub const OCR_DEFERRED: &str = "Image and scan OCR is not included. If a PDF page or a DOCX has no extractable text, the report says so and stops. Figures and charts are not interpreted. No external OCR program is called.";

pub const SET_ASIDE_DESC: &str = "A passage is set aside when it is mostly a quotation, when it follows a references or bibliography heading, or when it matches a file passed with --boilerplate. Set-aside overlap is still 4-token shingle overlap. It is not a semantic reading, and it is not listed as the main overlap evidence.";

pub const SET_ASIDE_QUOTE: &str = "Set aside because the passage is mostly a quotation or a Markdown block quote. Shared quotations are not treated as evidence of copied prose. This is still 4-token shingle overlap, not a reading of meaning.";

pub const SET_ASIDE_BIB: &str = "Set aside because the passage sits under a references or bibliography heading. Matching references are not treated as evidence of copied prose. This is still 4-token shingle overlap, not a reading of meaning.";

pub const SET_ASIDE_TEMPLATE: &str = "Set aside because the passage matches a file passed with --boilerplate. Handout and template wording is not treated as evidence of copied prose. This is still 4-token shingle overlap, not a reading of meaning.";

pub const DB_COMPAT: &str = "Database schema version stays 1 in this release. A 0.1.0 database opens without migration. DOCX and PDF bytes are not stored; only the extracted plain text is stored, and only when a passage is long enough to compare.";

pub const LIMITATIONS: &[&str] = &[
    "DOCX and text-based PDF are extracted to plain text. Legacy .doc, scanned pages, images, and charts are not read. OCR is not included.",
    "No trained model is included, and none of the figures is a probability of AI use.",
    "Chinese text is tokenized per character. There is no dictionary word segmenter.",
    "Near-exact matching uses 4-token shingles. Paraphrase, summary, and translation are not detected.",
    "Sentence boundaries are heuristic. A small set of abbreviations and decimal points is handled; other dots can split or join sentences.",
    "Short text makes length, variation, and vocabulary figures unstable. The report says so when that is the case.",
    "Technical lab prose and shared assignment wording can look repetitive or similar without being misconduct.",
    "A match shows overlap with a document you added locally. It does not show who copied whom, or that a rule was broken.",
    "Digits split mixed tokens, so a formula such as H2O becomes separate tokens. Chemistry writeups can look less diverse for that reason alone.",
];

pub fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docs_quote_the_method_strings() {
        let methodology = include_str!("../docs/methodology.md");
        let readme = include_str!("../README.md");
        let readme_zh = include_str!("../README.zh-CN.md");
        let privacy = include_str!("../docs/privacy.md");
        for required in [
            TOKENIZER_DESC,
            COUNT_STREAMS,
            SHINGLE_DESC,
            CONTAINMENT_DENOMINATOR,
            JACCARD_DESC,
            UNICODE_DESC,
            EXACT_DESC,
            HASH_DESC,
            SELF_DESC,
            LENGTH_UNIT_NOTE,
            VOCAB_NOTE,
            REPETITION_NOTE,
            TRANSITION_NOTE,
            PERSISTED_TEXT_NOTE,
            SCOPE_LINE,
            DISCLAIMER,
            NO_OVERALL_SCORE,
            EXCERPT_NOTE,
            EXTRACTION_DESC,
            OCR_DEFERRED,
            SET_ASIDE_DESC,
            DB_COMPAT,
        ] {
            assert!(
                methodology.contains(required),
                "methodology.md is missing a method string"
            );
        }
        assert!(readme.contains(DISCLAIMER));
        assert!(readme_zh.contains(DISCLAIMER));
        assert!(privacy.contains(PERSISTED_TEXT_NOTE));
        assert_eq!(SHINGLE_K, 4);
        assert_eq!(MIN_SHARED_SHINGLES, 4);
        assert_eq!(MIN_NORMALIZED_CHARS, 40);
        assert_eq!(DB_SCHEMA_VERSION, "1");
    }
}
