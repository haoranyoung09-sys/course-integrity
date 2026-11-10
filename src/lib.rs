//! Local-first coursework integrity precheck.
//!
//! Reports are descriptive review signals for undergraduate drafts.
//! They are not an authorship test, not an AI probability, and not a plagiarism verdict.

#![forbid(unsafe_code)]

pub mod cli;
mod corpus;
mod error;
mod extract;
mod files;
mod html;
mod method;
mod regions;
mod report;
mod stats;
mod tokenize;

pub use corpus::{
    boilerplate_passages, threshold_basis_points, AddOutcome, BoilerplatePassage, CheckOptions,
    Coverage, Database, DocInfo,
};
pub use error::Error;
pub use method::{
    AUTHORSHIP_LINE, CONTAINMENT_DENOMINATOR, COUNT_STREAMS, DISCLAIMER, EXACT_DESC, EXCERPT_NOTE,
    HASH_DESC, JACCARD_DESC, LENGTH_UNIT_NOTE, NOT_PLAGIARISM_LINE, NO_OVERALL_SCORE,
    PERSISTED_TEXT_NOTE, REPETITION_NOTE, SCOPE_LINE, SELF_DESC, SHINGLE_DESC, TOKENIZER_DESC,
    TRANSITION_NOTE, UNICODE_DESC, VOCAB_NOTE,
};
pub use tokenize::{
    normalize_exact, score_shingles, shingles, tokenize, ShingleScore, Token, TokenKind,
};
