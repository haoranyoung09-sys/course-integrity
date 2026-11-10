use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::error::Error;
use crate::method::{
    BELOW_THRESHOLD_NOTE, CONTAINMENT_DENOMINATOR, DB_SCHEMA_VERSION, DOCUMENT_EXACT_NOTE,
    EXCERPT_NOTE, HASH_DESC, MAX_REPORTED_MATCHES, MIN_NORMALIZED_CHARS, MIN_SHARED_SHINGLES,
    NOT_PLAGIARISM_LINE, PERSISTED_TEXT_NOTE, SCOPE_LINE, SELF_DESC, SET_ASIDE_BIB,
    SET_ASIDE_QUOTE, SET_ASIDE_TEMPLATE,
};
use crate::regions::{paragraph_roles, PassageRole};
use crate::stats::{analyze, preview};
use crate::tokenize::{canonical_label, score_shingles};

const SCHEMA_SQL: &str = "
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY,
    source_label TEXT NOT NULL,
    content_hash TEXT NOT NULL UNIQUE,
    char_count INTEGER NOT NULL,
    passage_count INTEGER NOT NULL,
    added_at TEXT NOT NULL,
    text TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS passages (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    paragraph_index INTEGER NOT NULL,
    text TEXT NOT NULL,
    normalized TEXT NOT NULL,
    norm_hash TEXT NOT NULL,
    shingles_json TEXT NOT NULL,
    token_count INTEGER NOT NULL,
    comparable INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_passages_document ON passages(document_id);
CREATE INDEX IF NOT EXISTS idx_passages_norm_hash ON passages(norm_hash);
";

#[derive(Debug, Clone, Serialize)]
pub struct AddOutcome {
    pub source_label: String,
    pub status: String,
    pub document_id: Option<i64>,
    pub existing_label: Option<String>,
    pub characters: usize,
    pub stored_passages: usize,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocInfo {
    pub id: i64,
    pub source_label: String,
    pub content_hash: String,
    pub characters: usize,
    pub passages: usize,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Coverage {
    pub document_count: usize,
    pub passage_count: usize,
    pub comparable_passage_count: usize,
    pub total_characters: usize,
    pub scope: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExcludedSelf {
    pub document_id: i64,
    pub source_label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentExact {
    pub source_document_id: i64,
    pub source_label: String,
    pub note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchHit {
    pub query_paragraph_index: usize,
    pub source_document_id: i64,
    pub source_label: String,
    pub source_paragraph_index: usize,
    pub exact: bool,
    pub shared_shingles: usize,
    pub query_shingles: usize,
    pub source_shingles: usize,
    pub containment_basis_points: Option<u32>,
    pub jaccard_basis_points: Option<u32>,
    pub containment: Option<f64>,
    pub jaccard: Option<f64>,
    pub meets_threshold: bool,
    pub source_kind: String,
    pub set_aside: Option<&'static str>,
    pub score_note: &'static str,
    pub interpretation: &'static str,
    pub query_excerpt: String,
    pub source_excerpt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorpusSection {
    pub queried: bool,
    pub database_path: String,
    pub min_containment_basis_points: u32,
    pub min_shared_shingles: usize,
    pub coverage: Coverage,
    pub excluded_self: Vec<ExcludedSelf>,
    pub document_exact_matches: Vec<DocumentExact>,
    pub matches: Vec<MatchHit>,
    pub context_overlaps: Vec<MatchHit>,
    pub strongest_below_threshold: Option<MatchHit>,
    pub truncated_match_count: usize,
    pub truncated_context_count: usize,
    pub notes: Vec<String>,
    pub persisted_text_note: &'static str,
    pub self_comparison: &'static str,
    pub excerpt_note: &'static str,
    pub hash_note: &'static str,
}

#[derive(Debug, Clone)]
pub struct CheckOptions {
    pub min_containment_basis_points: u32,
    pub include_self: bool,
    pub boilerplate: Vec<BoilerplatePassage>,
}

#[derive(Debug, Clone)]
pub struct BoilerplatePassage {
    pub label: String,
    pub paragraph_index: usize,
    pub text: String,
    pub normalized: String,
    pub shingles: BTreeSet<String>,
    pub comparable: bool,
    pub exact_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveOutcome {
    pub document_id: i64,
    pub source_label: String,
    pub detail: String,
}

pub fn boilerplate_passages(label: &str, text: &str) -> Vec<BoilerplatePassage> {
    crate::tokenize::passage_drafts(text)
        .into_iter()
        .filter(|passage| passage.exact_eligible)
        .map(|passage| BoilerplatePassage {
            label: canonical_label(label),
            paragraph_index: passage.index,
            text: passage.text,
            normalized: passage.normalized,
            shingles: passage.shingles,
            comparable: passage.comparable,
            exact_eligible: passage.exact_eligible,
        })
        .collect()
}

pub struct Database {
    path: PathBuf,
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, Error> {
        if path.is_dir() {
            return Err(Error::Database {
                path: path.to_path_buf(),
                message: "database path is a directory".to_string(),
            });
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|source| Error::Io {
                    path: Some(parent.to_path_buf()),
                    source,
                })?;
            }
        }
        let conn = Connection::open(path).map_err(|err| db_err(path, err))?;
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|err| db_err(path, err))?;
        conn.execute_batch(SCHEMA_SQL)
            .map_err(|err| db_err(path, err))?;
        ensure_schema(&conn, path)?;
        Ok(Self {
            path: path.to_path_buf(),
            conn,
        })
    }

    pub fn path_display(&self) -> String {
        self.path.display().to_string()
    }

    pub fn add_text(&mut self, label: &str, text: &str) -> Result<AddOutcome, Error> {
        let label = canonical_label(label);
        let characters = text.chars().count();
        let analysis = analyze(text);
        if !analysis
            .passages
            .iter()
            .any(|passage| passage.exact_eligible)
        {
            return Ok(AddOutcome {
                source_label: label,
                status: "skipped".to_string(),
                document_id: None,
                existing_label: None,
                characters,
                stored_passages: 0,
                detail: format!(
                    "No passage reached {MIN_NORMALIZED_CHARS} normalized characters, so the file was not stored."
                ),
            });
        }

        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_err(&self.path, err))?;
        let existing: Option<(i64, String)> = tx
            .query_row(
                "SELECT id, source_label FROM documents WHERE content_hash = ?1",
                [&analysis.content_hash],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|err| db_err(&self.path, err))?;
        if let Some((id, existing_label)) = existing {
            return Ok(AddOutcome {
                source_label: label,
                status: "duplicate".to_string(),
                document_id: Some(id),
                existing_label: Some(existing_label.clone()),
                characters,
                stored_passages: 0,
                detail: format!(
                    "An identical normalized document is already stored as id {id} ({existing_label}). A second copy was not inserted. Checking a different label with this content still reports a match against the stored copy."
                ),
            });
        }

        let passage_count = analysis.passages.len() as i64;
        tx.execute(
            "INSERT INTO documents (source_label, content_hash, char_count, passage_count, added_at, text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                label,
                analysis.content_hash,
                i64_from_usize(characters)?,
                passage_count,
                now_rfc3339(),
                text
            ],
        )
        .map_err(|err| db_err(&self.path, err))?;
        let id = tx.last_insert_rowid();
        for passage in &analysis.passages {
            let shingles = serde_json::to_string(&passage.shingles.iter().collect::<Vec<_>>())
                .map_err(|err| db_err(&self.path, err))?;
            tx.execute(
                "INSERT INTO passages (
                    document_id, paragraph_index, text, normalized, norm_hash, shingles_json, token_count, comparable
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    i64_from_usize(passage.index)?,
                    passage.text,
                    passage.normalized,
                    passage.norm_hash,
                    shingles,
                    i64_from_usize(passage.tokens.len())?,
                    i64::from(u8::from(passage.comparable))
                ],
            )
            .map_err(|err| db_err(&self.path, err))?;
        }
        tx.commit().map_err(|err| db_err(&self.path, err))?;
        Ok(AddOutcome {
            source_label: label,
            status: "inserted".to_string(),
            document_id: Some(id),
            existing_label: None,
            characters,
            stored_passages: analysis.passages.len(),
            detail: format!(
                "Stored the full original text locally, with {} passage(s).",
                analysis.passages.len()
            ),
        })
    }

    pub fn list_documents(&self) -> Result<Vec<DocInfo>, Error> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source_label, content_hash, char_count, passage_count, added_at
                 FROM documents ORDER BY id",
            )
            .map_err(|err| db_err(&self.path, err))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DocInfo {
                    id: row.get(0)?,
                    source_label: row.get(1)?,
                    content_hash: row.get(2)?,
                    characters: usize_from_i64(row.get(3)?),
                    passages: usize_from_i64(row.get(4)?),
                    added_at: row.get(5)?,
                })
            })
            .map_err(|err| db_err(&self.path, err))?;
        let mut documents = Vec::new();
        for row in rows {
            documents.push(row.map_err(|err| db_err(&self.path, err))?);
        }
        Ok(documents)
    }

    pub fn document_text(&self, id: i64) -> Result<String, Error> {
        match self
            .conn
            .query_row("SELECT text FROM documents WHERE id = ?1", [id], |row| {
                row.get(0)
            }) {
            Ok(text) => Ok(text),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::Database {
                path: self.path.clone(),
                message: format!("no document with id {id}"),
            }),
            Err(err) => Err(db_err(&self.path, err)),
        }
    }

    pub fn coverage(&self) -> Result<Coverage, Error> {
        let (documents, characters, passages): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(char_count), 0), COALESCE(SUM(passage_count), 0) FROM documents",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|err| db_err(&self.path, err))?;
        let comparable: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM passages WHERE comparable = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|err| db_err(&self.path, err))?;
        Ok(Coverage {
            document_count: usize_from_i64(documents),
            passage_count: usize_from_i64(passages),
            comparable_passage_count: usize_from_i64(comparable),
            total_characters: usize_from_i64(characters),
            scope: SCOPE_LINE,
        })
    }

    pub fn check_text(
        &self,
        label: &str,
        text: &str,
        options: &CheckOptions,
    ) -> Result<CorpusSection, Error> {
        let label = canonical_label(label);
        let analysis = analyze(text);
        let coverage = self.coverage()?;
        let documents = self.load_documents()?;
        let stored = self.load_passages()?;
        let mut excluded_self = Vec::new();
        let mut excluded_ids = BTreeSet::new();
        if !options.include_self {
            for document in &documents {
                if document.label == label {
                    excluded_ids.insert(document.id);
                    excluded_self.push(ExcludedSelf {
                        document_id: document.id,
                        source_label: document.label.clone(),
                        reason: format!(
                            "Stored document {} uses the same source label ({}). It was excluded so this file is not reported as overlapping its own stored copy. Pass --include-self to compare them.",
                            document.id, document.label
                        ),
                    });
                }
            }
        }

        let document_exact_matches = documents
            .iter()
            .filter(|document| {
                document.hash == analysis.content_hash && !excluded_ids.contains(&document.id)
            })
            .map(|document| DocumentExact {
                source_document_id: document.id,
                source_label: document.label.clone(),
                note: DOCUMENT_EXACT_NOTE,
            })
            .collect::<Vec<_>>();

        let mut notes = vec![PERSISTED_TEXT_NOTE.to_string(), SCOPE_LINE.to_string()];
        if options.include_self {
            notes.push(
                "Self-matches are included because --include-self was set. A hit against the same source label is the stored copy, not an independent source.".to_string(),
            );
        }
        if coverage.document_count == 0 {
            notes.push(
                "The local corpus has no stored documents. No overlap can be found. That is not evidence that the draft is original.".to_string(),
            );
        }
        let query_comparable = analysis
            .passages
            .iter()
            .any(|passage| passage.exact_eligible);
        if !query_comparable {
            notes.push(format!(
                "The query has no passage of at least {MIN_NORMALIZED_CHARS} normalized characters, so there is nothing to compare. No match is not evidence of originality."
            ));
        }

        let roles = paragraph_roles(text);
        let mut meeting = Vec::new();
        let mut context = Vec::new();
        let mut strongest: Option<MatchHit> = None;
        if query_comparable {
            for query in &analysis.passages {
                if !query.exact_eligible {
                    continue;
                }
                let structural = roles
                    .get(query.index.saturating_sub(1))
                    .copied()
                    .unwrap_or(PassageRole::Prose);
                let template = best_boilerplate(
                    query,
                    &options.boilerplate,
                    options.min_containment_basis_points,
                );
                let set_aside = if template.is_some() {
                    Some(SET_ASIDE_TEMPLATE)
                } else {
                    match structural {
                        PassageRole::Prose => None,
                        PassageRole::Quotation => Some(SET_ASIDE_QUOTE),
                        PassageRole::Bibliography => Some(SET_ASIDE_BIB),
                    }
                };
                if let Some(plate) = template {
                    context.push(boilerplate_hit(query, plate));
                }
                for source in &stored {
                    if excluded_ids.contains(&source.document_id) {
                        continue;
                    }
                    let exact = source.normalized == query.normalized
                        && source.normalized.chars().count() >= MIN_NORMALIZED_CHARS;
                    let scored = score_shingles(&query.shingles, &source.shingles);
                    let containment_points = scored.containment_basis_points.unwrap_or(0);
                    let near = query.comparable
                        && source.comparable
                        && scored.shared >= MIN_SHARED_SHINGLES
                        && containment_points >= options.min_containment_basis_points;
                    let meets = exact || near;
                    if !meets && scored.shared == 0 {
                        continue;
                    }
                    let hit = MatchHit {
                        query_paragraph_index: query.index,
                        source_document_id: source.document_id,
                        source_label: source.label.clone(),
                        source_paragraph_index: source.paragraph_index,
                        exact,
                        shared_shingles: scored.shared,
                        query_shingles: scored.query_shingles,
                        source_shingles: scored.source_shingles,
                        containment_basis_points: scored.containment_basis_points,
                        jaccard_basis_points: scored.jaccard_basis_points,
                        containment: scored.containment,
                        jaccard: scored.jaccard,
                        meets_threshold: meets,
                        source_kind: "corpus".to_string(),
                        set_aside,
                        score_note: CONTAINMENT_DENOMINATOR,
                        interpretation: if let Some(reason) = set_aside {
                            reason
                        } else if meets {
                            NOT_PLAGIARISM_LINE
                        } else {
                            BELOW_THRESHOLD_NOTE
                        },
                        query_excerpt: preview(&query.text, 400),
                        source_excerpt: preview(&source.text, 400),
                    };
                    if meets && set_aside.is_none() {
                        meeting.push(hit);
                    } else if meets {
                        context.push(hit);
                    } else if set_aside.is_none()
                        && strongest.as_ref().is_none_or(|current| {
                            hit.containment_basis_points.unwrap_or(0)
                                > current.containment_basis_points.unwrap_or(0)
                        })
                    {
                        strongest = Some(hit);
                    }
                }
            }
        }

        sort_hits(&mut meeting);
        sort_hits(&mut context);
        let truncated_match_count = meeting.len().saturating_sub(MAX_REPORTED_MATCHES);
        let truncated_context_count = context.len().saturating_sub(MAX_REPORTED_MATCHES);
        meeting.truncate(MAX_REPORTED_MATCHES);
        context.truncate(MAX_REPORTED_MATCHES);
        if coverage.document_count > 0
            && meeting.is_empty()
            && context.is_empty()
            && query_comparable
        {
            notes.push(
                "No passage met the exact or near-exact threshold against the stored passages that were compared.".to_string(),
            );
        }
        if !context.is_empty() {
            notes.push(
                "Some overlap was set aside because the passage is a quotation, a bibliography section, or boilerplate supplied with --boilerplate. Those rows are not treated as evidence of copied prose. The comparison is still shingle overlap, not a semantic reading.".to_string(),
            );
        }
        if !excluded_self.is_empty() && meeting.is_empty() {
            notes.push(
                "Stored rows with this source label were excluded. This file was not compared with its own stored copy.".to_string(),
            );
        }

        Ok(CorpusSection {
            queried: true,
            database_path: self.path_display(),
            min_containment_basis_points: options.min_containment_basis_points,
            min_shared_shingles: MIN_SHARED_SHINGLES,
            coverage,
            excluded_self,
            document_exact_matches,
            matches: meeting,
            context_overlaps: context,
            strongest_below_threshold: strongest,
            truncated_match_count,
            truncated_context_count,
            notes,
            persisted_text_note: PERSISTED_TEXT_NOTE,
            self_comparison: SELF_DESC,
            excerpt_note: EXCERPT_NOTE,
            hash_note: HASH_DESC,
        })
    }

    pub fn schema_version(&self) -> Result<String, Error> {
        self.conn
            .query_row("SELECT value FROM meta WHERE key = 'schema'", [], |row| {
                row.get(0)
            })
            .map_err(|err| db_err(&self.path, err))
    }

    pub fn remove_document(&mut self, id: i64) -> Result<RemoveOutcome, Error> {
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_err(&self.path, err))?;
        let existed: Option<String> = tx
            .query_row(
                "SELECT source_label FROM documents WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| db_err(&self.path, err))?;
        let Some(source_label) = existed else {
            return Err(Error::Database {
                path: self.path.clone(),
                message: format!(
                    "no document with id {id}. `corpus list` shows ids. Nothing was deleted."
                ),
            });
        };
        tx.execute("DELETE FROM documents WHERE id = ?1", [id])
            .map_err(|err| db_err(&self.path, err))?;
        tx.commit().map_err(|err| db_err(&self.path, err))?;
        Ok(RemoveOutcome {
            document_id: id,
            source_label,
            detail: "Deleted the stored text and its passages from this local database. No remote copy exists.".to_string(),
        })
    }

    fn load_documents(&self) -> Result<Vec<StoredDocument>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, source_label, content_hash FROM documents ORDER BY id")
            .map_err(|err| db_err(&self.path, err))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(StoredDocument {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    hash: row.get(2)?,
                })
            })
            .map_err(|err| db_err(&self.path, err))?;
        let mut documents = Vec::new();
        for row in rows {
            documents.push(row.map_err(|err| db_err(&self.path, err))?);
        }
        Ok(documents)
    }

    fn load_passages(&self) -> Result<Vec<StoredPassage>, Error> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.document_id, d.source_label, p.paragraph_index, p.text, p.normalized, p.shingles_json, p.comparable
                 FROM passages p
                 JOIN documents d ON d.id = p.document_id
                 ORDER BY p.document_id, p.paragraph_index",
            )
            .map_err(|err| db_err(&self.path, err))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })
            .map_err(|err| db_err(&self.path, err))?;
        let mut passages = Vec::new();
        for row in rows {
            let (document_id, label, paragraph_index, text, normalized, shingles_json, comparable) =
                row.map_err(|err| db_err(&self.path, err))?;
            let list: Vec<String> =
                serde_json::from_str(&shingles_json).map_err(|err| Error::Database {
                    path: self.path.clone(),
                    message: format!("could not read stored shingles: {err}"),
                })?;
            passages.push(StoredPassage {
                document_id,
                label,
                paragraph_index: usize_from_i64(paragraph_index),
                text,
                normalized,
                shingles: list.into_iter().collect(),
                comparable: comparable != 0,
            });
        }
        Ok(passages)
    }
}

struct StoredDocument {
    id: i64,
    label: String,
    hash: String,
}

struct StoredPassage {
    document_id: i64,
    label: String,
    paragraph_index: usize,
    text: String,
    normalized: String,
    shingles: BTreeSet<String>,
    comparable: bool,
}

pub fn threshold_basis_points(value: f64) -> Result<u32, Error> {
    if !value.is_finite() || value <= 0.0 || value > 1.0 {
        return Err(Error::BadThreshold(value));
    }
    let points = (value * 10_000.0).round();
    if !(1.0..=10_000.0).contains(&points) {
        return Err(Error::BadThreshold(value));
    }
    Ok(points as u32)
}

fn best_boilerplate<'a>(
    query: &crate::tokenize::PassageDraft,
    plates: &'a [BoilerplatePassage],
    threshold: u32,
) -> Option<&'a BoilerplatePassage> {
    let mut best: Option<(&BoilerplatePassage, u32)> = None;
    for plate in plates {
        let exact = query.exact_eligible
            && plate.exact_eligible
            && query.normalized == plate.normalized
            && query.normalized.chars().count() >= MIN_NORMALIZED_CHARS;
        let scored = score_shingles(&query.shingles, &plate.shingles);
        let points = scored.containment_basis_points.unwrap_or(0);
        let near = query.comparable
            && plate.comparable
            && scored.shared >= MIN_SHARED_SHINGLES
            && points >= threshold;
        if exact || near {
            let rank = if exact { 10_000 } else { points };
            if best.is_none_or(|(_, current)| rank > current) {
                best = Some((plate, rank));
            }
        }
    }
    best.map(|(plate, _)| plate)
}

fn boilerplate_hit(query: &crate::tokenize::PassageDraft, plate: &BoilerplatePassage) -> MatchHit {
    let exact = query.normalized == plate.normalized;
    let scored = score_shingles(&query.shingles, &plate.shingles);
    MatchHit {
        query_paragraph_index: query.index,
        source_document_id: 0,
        source_label: plate.label.clone(),
        source_paragraph_index: plate.paragraph_index,
        exact,
        shared_shingles: scored.shared,
        query_shingles: scored.query_shingles,
        source_shingles: scored.source_shingles,
        containment_basis_points: scored.containment_basis_points,
        jaccard_basis_points: scored.jaccard_basis_points,
        containment: scored.containment,
        jaccard: scored.jaccard,
        meets_threshold: true,
        source_kind: "boilerplate".to_string(),
        set_aside: Some(SET_ASIDE_TEMPLATE),
        score_note: CONTAINMENT_DENOMINATOR,
        interpretation: SET_ASIDE_TEMPLATE,
        query_excerpt: preview(&query.text, 400),
        source_excerpt: preview(&plate.text, 400),
    }
}

fn sort_hits(hits: &mut [MatchHit]) {
    hits.sort_by(|left, right| {
        right
            .exact
            .cmp(&left.exact)
            .then_with(|| {
                right
                    .containment_basis_points
                    .unwrap_or(0)
                    .cmp(&left.containment_basis_points.unwrap_or(0))
            })
            .then_with(|| left.query_paragraph_index.cmp(&right.query_paragraph_index))
            .then_with(|| left.source_document_id.cmp(&right.source_document_id))
            .then_with(|| {
                left.source_paragraph_index
                    .cmp(&right.source_paragraph_index)
            })
    });
}

fn ensure_schema(conn: &Connection, path: &Path) -> Result<(), Error> {
    let found: Option<String> = conn
        .query_row("SELECT value FROM meta WHERE key = 'schema'", [], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|err| db_err(path, err))?;
    match found.as_deref() {
        None => {
            conn.execute(
                "INSERT INTO meta (key, value) VALUES ('schema', ?1)",
                [DB_SCHEMA_VERSION],
            )
            .map_err(|err| db_err(path, err))?;
            Ok(())
        }
        Some(version) if version == DB_SCHEMA_VERSION => Ok(()),
        Some(version) => Err(Error::Schema {
            path: path.to_path_buf(),
            found: version.to_string(),
        }),
    }
}

fn db_err(path: &Path, err: impl ToString) -> Error {
    Error::Database {
        path: path.to_path_buf(),
        message: err.to_string(),
    }
}

fn i64_from_usize(value: usize) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Database {
        path: PathBuf::new(),
        message: format!("value {value} does not fit in the database integer column"),
    })
}

fn usize_from_i64(value: i64) -> usize {
    usize::try_from(value).unwrap_or(0)
}

fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format_unix(secs)
}

fn format_unix(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn civil_from_days(z_in: i64) -> (i64, u32, u32) {
    let z = z_in + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u64::try_from(z - era * 146_097).unwrap_or(0);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = i64::try_from(yoe).unwrap_or(0) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    if month <= 2 {
        year += 1;
    }
    (
        year,
        u32::try_from(month).unwrap_or(1),
        u32::try_from(day).unwrap_or(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_timestamps_format_as_utc() {
        assert_eq!(format_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_unix(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(format_unix(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn threshold_rejects_zero_and_accepts_default() {
        assert!(threshold_basis_points(0.0).is_err());
        assert!(threshold_basis_points(1.1).is_err());
        assert_eq!(threshold_basis_points(0.8).unwrap(), 8000);
        assert_eq!(threshold_basis_points(1.0).unwrap(), 10_000);
    }
}
