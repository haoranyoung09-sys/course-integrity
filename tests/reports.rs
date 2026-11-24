use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_course_integrity").unwrap_or_else(|_| {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target/debug/course-integrity");
        path.to_string_lossy().into_owned()
    })
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run course-integrity")
}

#[test]
fn html_escapes_markup_and_all_format_needs_a_directory() {
    let dir = std::env::temp_dir().join(format!("course-integrity-html-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let input = dir.join("inject.txt");
    fs::write(
        &input,
        "Before <script>alert(1)</script> after, the sentence stays visible in the report.\n",
    )
    .unwrap();
    let html_run = run(&["scan", input.to_str().unwrap(), "--format", "html"]);
    assert!(
        html_run.status.success(),
        "{}",
        String::from_utf8_lossy(&html_run.stderr)
    );
    let html = String::from_utf8_lossy(&html_run.stdout);
    assert!(html.contains("Before &lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("no overall score") || html.contains("There is no overall score"));
    assert!(!html.contains("http://") && !html.contains("https://"));

    let missing_dir = run(&["scan", input.to_str().unwrap(), "--format", "all"]);
    assert_eq!(missing_dir.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing_dir.stderr).contains("--out-dir"));

    let out = dir.join("out");
    let all = run(&[
        "scan",
        input.to_str().unwrap(),
        "--format",
        "all",
        "--out-dir",
        out.to_str().unwrap(),
    ]);
    assert!(
        all.status.success(),
        "{}",
        String::from_utf8_lossy(&all.stderr)
    );
    let stderr = String::from_utf8_lossy(&all.stderr);
    assert!(stderr.contains(".review.md"));
    assert!(stderr.contains(".review.html"));
    assert!(stderr.contains(".review.json"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn docx_and_pdf_smoke_mentions_locations_and_limits() {
    let docx = run(&[
        "scan",
        "fixtures/synthetic/lab-report-zh.docx",
        "--format",
        "json",
    ]);
    assert!(
        docx.status.success(),
        "{}",
        String::from_utf8_lossy(&docx.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&docx.stdout).unwrap();
    let file = &value["files"][0];
    assert_eq!(file["origin"], "docx");
    assert!(
        file["document"]["token_counts"]["cjk_or_kana"]
            .as_u64()
            .unwrap()
            > 10
    );
    let notes = file["extraction_notes"].to_string();
    assert!(notes.contains("OCR is not included"));
    assert!(notes.contains("embedded image") || notes.contains("paragraph"));
    assert!(file["paragraph_locations"][0]
        .as_str()
        .unwrap()
        .contains("word/document.xml"));

    let pdf = run(&[
        "scan",
        "fixtures/synthetic/lab-report-en.pdf",
        "--format",
        "json",
    ]);
    assert!(
        pdf.status.success(),
        "{}",
        String::from_utf8_lossy(&pdf.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&pdf.stdout).unwrap();
    let file = &value["files"][0];
    assert_eq!(file["origin"], "pdf");
    assert!(file["paragraph_locations"][0]
        .as_str()
        .unwrap()
        .starts_with("page 1"));
    assert!(file["extraction_notes"]
        .to_string()
        .contains("OCR is not included"));
}

#[test]
fn hidden_marks_are_located_and_the_file_is_not_rewritten() {
    let path = "fixtures/synthetic/hidden-marks.txt";
    let before = fs::read(path).unwrap();
    let output = run(&["scan", path, "--format", "json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(path).unwrap(), before);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let artifacts = &value["files"][0]["artifacts"];
    assert_eq!(artifacts["newline_style"], "mixed");
    let findings = artifacts["findings"].as_array().unwrap();
    assert!(findings
        .iter()
        .any(|item| item["kind"] == "crlf" && item["line"] == 1));
    assert!(findings.iter().any(|item| item["kind"] == "nbsp"));
    assert!(findings.iter().any(|item| item["kind"] == "zero_width"));
    assert!(findings.iter().any(|item| item["code_point"] == "U+202E"));
    let html = run(&["scan", path, "--format", "html"]);
    assert!(html.status.success());
    let page = String::from_utf8_lossy(&html.stdout);
    assert!(!page.contains('\u{202E}'));
    assert!(page.contains("U+202E"));
    assert!(page.contains("not an AI-generation rate"));
}

#[test]
fn synthetic_batch_of_forty_files_finishes_quickly_in_debug() {
    let dir = std::env::temp_dir().join(format!("course-integrity-batch-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for index in 0..40 {
        fs::write(
            dir.join(format!("note-{index}.md")),
            format!(
                "Synthetic fixture {index}.\n\nThe beaker number {index} cooled on the bench and the group recorded the temperature every thirty seconds without fitting a model or measuring the room.\n"
            ),
        )
        .unwrap();
    }
    let started = Instant::now();
    let output = run(&["scan", dir.to_str().unwrap(), "--format", "json"]);
    let elapsed = started.elapsed();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["files"].as_array().unwrap().len(), 40);
    assert!(
        elapsed.as_secs() < 20,
        "debug scan of 40 synthetic files took {elapsed:?}; this is a ceiling, not a benchmark"
    );
    eprintln!(
        "measurement: debug scan of 40 synthetic markdown files took {elapsed:?} on this machine"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn html_and_markdown_include_corpus_details_and_locations() {
    let dir = std::env::temp_dir().join(format!("course-integrity-rich-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let shared = "In this synthetic cooling exercise the group recorded the temperature of two hundred millilitres of water in a glass beaker carefully.";
    let near_a = "However, we measured the room temperature every thirty seconds on the open bench, and the water cooled steadily from sixty degrees to forty nine degrees.";
    let near_b = "However, we measured the room temperature every thirty seconds on the open bench, and the liquid cooled slowly from sixty degrees to forty five degrees.";
    fs::write(
        dir.join("alpha.md"),
        format!("Synthetic fixture.\n\n{shared}\n\n{near_a}\n"),
    )
    .unwrap();
    fs::write(
        dir.join("beta.md"),
        format!("Synthetic fixture.\n\n{shared}\n\n{near_b}\n"),
    )
    .unwrap();

    let html_run = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--pairwise",
        "--format",
        "html",
    ]);
    assert!(
        html_run.status.success(),
        "{}",
        String::from_utf8_lossy(&html_run.stderr)
    );
    let html = String::from_utf8_lossy(&html_run.stdout);
    assert!(html.contains("<nav class=\"toc\""));
    assert!(html.contains("<div class=\"cards\">"));
    assert!(html.contains("<div class=\"compare\">"));
    assert!(html.contains("#file-1-p-"));
    assert!(html.contains("prefers-color-scheme: dark"));
    assert!(html.contains("Transition inventory"));
    assert!(html.contains("<h3>Repetition</h3>"));
    assert!(html.contains("Excluded as self"));
    assert!(html.contains("Nearest passage below the threshold"));
    assert!(html.contains("Transition phrases are a fixed inventory"));
    assert!(!html.contains("<script"));
    assert!(!html.contains("http://") && !html.contains("https://"));

    let md_docx = run(&[
        "scan",
        "fixtures/synthetic/lab-report-en.docx",
        "--format",
        "markdown",
    ]);
    assert!(md_docx.status.success());
    let md = String::from_utf8_lossy(&md_docx.stdout);
    assert!(md.contains("(word/document.xml paragraph"));

    let _ = fs::remove_dir_all(&dir);
}
