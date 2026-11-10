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
