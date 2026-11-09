use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use course_integrity::{CheckOptions, Database};

fn db_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "course-integrity-{name}-{}-{nanos}.db",
        std::process::id()
    ));
    path
}

fn read(relative: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {relative}: {err}"))
}

fn options(include_self: bool) -> CheckOptions {
    CheckOptions {
        min_containment_basis_points: 8000,
        include_self,
    }
}

#[test]
fn roundtrip_duplicate_self_and_english_chinese_matches() {
    let path = db_path("corpus");
    let mut db = Database::open(&path).expect("open db");
    let source = read("fixtures/synthetic/en_source.md");
    let added = db
        .add_text("fixtures/synthetic/en_source.md", &source)
        .expect("add source");
    assert_eq!(added.status, "inserted");
    let id = added.document_id.expect("id");
    assert_eq!(db.document_text(id).expect("text"), source);

    let duplicate = db
        .add_text("classmate-copy.md", &source)
        .expect("duplicate add");
    assert_eq!(duplicate.status, "duplicate");
    assert_eq!(duplicate.document_id, Some(id));
    assert_eq!(db.list_documents().expect("list").len(), 1);

    let skipped = db.add_text("short.txt", "Hi.").expect("skip short");
    assert_eq!(skipped.status, "skipped");
    assert!(skipped.document_id.is_none());
    assert_eq!(db.list_documents().expect("list").len(), 1);

    let excluded = db
        .check_text("fixtures/synthetic/en_source.md", &source, &options(false))
        .expect("self check");
    assert!(excluded.matches.is_empty());
    assert_eq!(excluded.excluded_self.len(), 1);
    assert!(excluded.persisted_text_note.contains("full original text"));

    let included = db
        .check_text("fixtures/synthetic/en_source.md", &source, &options(true))
        .expect("include self");
    assert!(included.matches.iter().any(|hit| {
        hit.exact && hit.containment_basis_points == Some(10_000) && hit.meets_threshold
    }));

    let other_label = db
        .check_text("classmate-copy.md", &source, &options(false))
        .expect("different label");
    assert!(other_label
        .document_exact_matches
        .iter()
        .any(|hit| hit.source_document_id == id));
    assert!(other_label.matches.iter().any(|hit| hit.exact));
    assert!(other_label
        .matches
        .iter()
        .all(|hit| hit.interpretation.contains("not a plagiarism verdict")));

    let exact = db
        .check_text(
            "fixtures/synthetic/en_exact_paragraph.md",
            &read("fixtures/synthetic/en_exact_paragraph.md"),
            &options(false),
        )
        .expect("exact paragraph");
    assert!(exact.matches.iter().any(|hit| {
        hit.exact
            && hit.query_paragraph_index == 2
            && hit.source_label.ends_with("en_source.md")
            && hit.containment_basis_points == Some(10_000)
    }));
    assert!(exact.document_exact_matches.is_empty());

    let near = db
        .check_text(
            "fixtures/synthetic/en_near_paragraph.md",
            &read("fixtures/synthetic/en_near_paragraph.md"),
            &options(false),
        )
        .expect("near paragraph");
    let near_hit = near
        .matches
        .iter()
        .find(|hit| hit.query_paragraph_index == 2)
        .expect("near hit");
    assert!(!near_hit.exact);
    assert!(near_hit.meets_threshold);
    let points = near_hit.containment_basis_points.expect("points");
    assert!((8000..10_000).contains(&points), "points {points}");
    assert!(near_hit.query_shingles > near_hit.shared_shingles);
    assert!(near_hit.score_note.contains("denominator"));

    let unrelated = db
        .check_text(
            "fixtures/synthetic/en_unrelated.md",
            &read("fixtures/synthetic/en_unrelated.md"),
            &options(false),
        )
        .expect("unrelated");
    assert!(unrelated.matches.is_empty());

    let zh_source = read("fixtures/synthetic/zh_source.md");
    let zh_added = db
        .add_text("fixtures/synthetic/zh_source.md", &zh_source)
        .expect("add zh");
    assert_eq!(zh_added.status, "inserted");
    assert_eq!(
        db.document_text(zh_added.document_id.expect("zh id"))
            .expect("zh text"),
        zh_source
    );
    let zh_exact = db
        .check_text(
            "fixtures/synthetic/zh_exact_paragraph.md",
            &read("fixtures/synthetic/zh_exact_paragraph.md"),
            &options(false),
        )
        .expect("zh exact");
    assert!(zh_exact.matches.iter().any(|hit| {
        hit.exact && hit.query_paragraph_index == 2 && hit.source_label.ends_with("zh_source.md")
    }));
    let zh_unrelated = db
        .check_text(
            "fixtures/synthetic/zh_unrelated.md",
            &read("fixtures/synthetic/zh_unrelated.md"),
            &options(false),
        )
        .expect("zh unrelated");
    assert!(zh_unrelated.matches.is_empty());

    let prompt = db
        .add_text(
            "fixtures/synthetic/shared_prompt.md",
            &read("fixtures/synthetic/shared_prompt.md"),
        )
        .expect("prompt");
    assert_eq!(prompt.status, "inserted");
    let group = db
        .check_text(
            "fixtures/synthetic/group_draft.md",
            &read("fixtures/synthetic/group_draft.md"),
            &options(false),
        )
        .expect("group");
    assert!(group.matches.iter().any(|hit| {
        hit.exact
            && hit.query_paragraph_index == 2
            && hit.source_label.ends_with("shared_prompt.md")
    }));

    let coverage = db.coverage().expect("coverage");
    assert_eq!(coverage.document_count, 3);
    assert!(coverage.scope.contains("not a commercial checker corpus"));
    assert!(coverage.total_characters > 0);

    drop(db);
    let _ = fs::remove_file(&path);
}

#[test]
fn empty_corpus_does_not_imply_originality() {
    let path = db_path("empty");
    let db = Database::open(&path).expect("open");
    let section = db
        .check_text(
            "draft.md",
            &read("fixtures/synthetic/en_source.md"),
            &options(false),
        )
        .expect("check");
    assert_eq!(section.coverage.document_count, 0);
    assert!(section.matches.is_empty());
    assert!(section
        .notes
        .iter()
        .any(|note| note.contains("no stored documents")));
    assert!(section
        .notes
        .iter()
        .any(|note| note.contains("not evidence that the draft is original")));
    drop(db);
    let _ = fs::remove_file(&path);
}
