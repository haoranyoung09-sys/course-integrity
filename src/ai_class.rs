use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};

use crate::method::{round4, AI_FLAGGED_SHARE, AI_NOT_RUN, AI_SCORE_DISTINCTION, AI_SCORE_LABEL};
use crate::tokenize::paragraphs;

pub const EN_REVISION: &str = "3ceb1d1bb7b9ca11585ce3f4299950e1c465b6d8";
pub const ZH_REVISION: &str = "47695ff451b32c225dd938f4f478f7fdc6aa6bb0";
const MAX_STDOUT: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct AiClassifierReport {
    pub status: String,
    pub reason: Option<String>,
    pub label: &'static str,
    pub distinction: &'static str,
    pub flagged_share_definition: &'static str,
    pub not_a_zero_result: &'static str,
    pub chunks: Vec<AiChunk>,
    pub scored_characters: usize,
    pub flagged_characters: usize,
    pub skipped_characters: usize,
    pub uncovered_characters: usize,
    pub flagged_character_share: Option<f64>,
    pub models: Vec<AiModelInfo>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiChunk {
    pub location: String,
    pub language: String,
    pub language_forced: bool,
    pub char_start: usize,
    pub char_end: usize,
    pub token_count: Option<usize>,
    pub scored: bool,
    pub reason: Option<String>,
    pub predicted_class: Option<String>,
    pub human_softmax: Option<f64>,
    pub ai_softmax: Option<f64>,
    pub score_0_to_100: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiModelInfo {
    pub key: String,
    pub repo: String,
    pub revision: String,
    pub weight_format: String,
}

#[derive(Clone)]
pub(crate) struct ClassifierOptions {
    pub enabled: bool,
    pub models_dir: PathBuf,
    pub script: PathBuf,
    pub python: String,
    pub language: String,
    pub timeout: Duration,
}

struct PreparedChunk {
    id: String,
    location: String,
    language: String,
    forced: bool,
    text: String,
}

pub(crate) fn not_run(reason: &str) -> AiClassifierReport {
    AiClassifierReport {
        status: "not_run".to_string(),
        reason: Some(reason.to_string()),
        label: AI_SCORE_LABEL,
        distinction: AI_SCORE_DISTINCTION,
        flagged_share_definition: AI_FLAGGED_SHARE,
        not_a_zero_result: AI_NOT_RUN,
        chunks: Vec::new(),
        scored_characters: 0,
        flagged_characters: 0,
        skipped_characters: 0,
        uncovered_characters: 0,
        flagged_character_share: None,
        models: Vec::new(),
        notes: vec![
            AI_SCORE_DISTINCTION.to_string(),
            AI_NOT_RUN.to_string(),
            "Short, mixed-language, academic, and non-native English text can be classified the wrong way. No accuracy figure is claimed.".to_string(),
        ],
    }
}

pub(crate) fn classify_text(
    text: &str,
    origin: &str,
    locations: &[String],
    options: &ClassifierOptions,
) -> AiClassifierReport {
    if !options.enabled {
        return not_run(
            "Pass --ai-classify to run the optional local model. No score was produced.",
        );
    }
    let mut report = classify_prepared(text, locations, options);
    if origin == "docx" || origin == "pdf" {
        report.notes.push(
            "The classifier saw extracted text only. Characters the extractor dropped are not scored, and they are not treated as 0.".to_string(),
        );
    }
    report
}

fn classify_prepared(
    text: &str,
    locations: &[String],
    options: &ClassifierOptions,
) -> AiClassifierReport {
    let prepared = prepare_chunks(text, locations, &options.language);
    if prepared.is_empty() {
        return not_run("empty_text");
    }
    if !options.script.is_file() {
        return not_run("missing_classifier_script");
    }
    let request = json!({
        "models_dir": options.models_dir,
        "chunks": prepared.iter().map(|chunk| json!({
            "id": chunk.id,
            "text": chunk.text,
            "language": chunk.language,
            "location": chunk.location,
        })).collect::<Vec<_>>(),
    });
    let raw = match run_sidecar(options, &request.to_string()) {
        Ok(raw) => raw,
        Err(reason) => return not_run(&reason),
    };
    match interpret(&prepared, &raw) {
        Ok(report) => report,
        Err(reason) => not_run(&reason),
    }
}

fn prepare_chunks(text: &str, locations: &[String], language_mode: &str) -> Vec<PreparedChunk> {
    paragraphs(text)
        .into_iter()
        .enumerate()
        .filter(|(_, paragraph)| !paragraph.trim().is_empty())
        .map(|(index, paragraph)| {
            let detected = detect_language(&paragraph);
            let (language, forced) = match language_mode {
                "en" | "zh" if detected == "mixed" || detected == "unknown" => {
                    (language_mode.to_string(), true)
                }
                "en" | "zh" if detected != language_mode => (language_mode.to_string(), true),
                _ => (detected.to_string(), false),
            };
            PreparedChunk {
                id: format!("p{}", index + 1),
                location: locations
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| format!("paragraph {}", index + 1)),
                language,
                forced,
                text: paragraph,
            }
        })
        .collect()
}

fn detect_language(text: &str) -> &'static str {
    let mut cjk = 0usize;
    let mut latin = 0usize;
    for ch in text.chars() {
        if is_cjk(ch) {
            cjk += 1;
        } else if ch.is_ascii_alphabetic() {
            latin += 1;
        }
    }
    let total = cjk + latin;
    if total == 0 {
        return "unknown";
    }
    let ratio = cjk as f64 / total as f64;
    if ratio >= 0.8 {
        "zh"
    } else if ratio <= 0.2 {
        "en"
    } else {
        "mixed"
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(ch, '\u{3400}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}')
}

fn run_sidecar(options: &ClassifierOptions, body: &str) -> Result<String, String> {
    let mut child = Command::new(&options.python)
        .arg(&options.script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("HF_HUB_OFFLINE", "1")
        .env("TRANSFORMERS_OFFLINE", "1")
        .env("HF_HUB_DISABLE_TELEMETRY", "1")
        .env_remove("HF_TOKEN")
        .env_remove("HUGGING_FACE_HUB_TOKEN")
        .spawn()
        .map_err(|_| "missing_python".to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(body.as_bytes())
            .map_err(|_| "sidecar_failed".to_string())?;
    }
    let mut stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| "sidecar_failed".to_string())?;
    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| "sidecar_failed".to_string())?;
    let stderr_thread = thread::spawn(move || {
        let mut discarded = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut discarded);
    });
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(read_capped_stdout(&mut stdout_pipe));
    });
    let started = Instant::now();
    let stdout = loop {
        let remaining = options
            .timeout
            .checked_sub(started.elapsed())
            .unwrap_or(Duration::ZERO);
        if remaining.is_zero() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stderr_thread.join();
            return Err("timeout".to_string());
        }
        match receiver.recv_timeout(remaining.min(Duration::from_millis(200))) {
            Ok(Ok(stdout)) => break stdout,
            Ok(Err(reason)) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stderr_thread.join();
                return Err(reason);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(Some(status)) = child.try_wait() {
                    if !status.success() {
                        let _ = stderr_thread.join();
                        return Err("sidecar_failed".to_string());
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stderr_thread.join();
                return Err("sidecar_failed".to_string());
            }
        }
    };
    let status = child.wait().map_err(|_| "sidecar_failed".to_string())?;
    let _ = stderr_thread.join();
    if !status.success() {
        return Err("sidecar_failed".to_string());
    }
    Ok(stdout)
}

fn read_capped_stdout(reader: &mut impl Read) -> Result<String, String> {
    let mut stdout = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(count) => {
                if stdout.len() + count > MAX_STDOUT {
                    return Err("sidecar_output_too_large".to_string());
                }
                stdout.extend_from_slice(&buf[..count]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("sidecar_failed".to_string()),
        }
    }
    String::from_utf8(stdout).map_err(|_| "invalid_output".to_string())
}

fn interpret(prepared: &[PreparedChunk], raw: &str) -> Result<AiClassifierReport, String> {
    let value: Value =
        serde_json::from_str(raw.trim()).map_err(|_| "invalid_output".to_string())?;
    let status = value.get("status").and_then(Value::as_str).unwrap_or("");
    if status == "not_run" {
        let reason = value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("not_run");
        return Ok(not_run(reason));
    }
    if status != "ok" {
        return Err("invalid_output".to_string());
    }
    let max_tokens = value.get("max_tokens").and_then(Value::as_u64).unwrap_or(0);
    if max_tokens != 512 {
        return Err("invalid_output".to_string());
    }
    let label_map = value.get("label_map").ok_or("invalid_output")?;
    if label_map.get("0").and_then(Value::as_str) != Some("human")
        || label_map.get("1").and_then(Value::as_str) != Some("ai")
    {
        return Err("invalid_output".to_string());
    }
    let models = parse_models(value.get("models").ok_or("invalid_output")?)?;
    let returned = value
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or("invalid_output")?;
    let mut by_parent: Vec<Vec<&Value>> = vec![Vec::new(); prepared.len()];
    for item in returned {
        let parent = item
            .get("parent_id")
            .and_then(Value::as_str)
            .ok_or("invalid_output")?;
        let index = prepared
            .iter()
            .position(|chunk| chunk.id == parent)
            .ok_or("invalid_output")?;
        by_parent[index].push(item);
    }
    let mut chunks = Vec::new();
    let mut scored_characters = 0usize;
    let mut flagged_characters = 0usize;
    let mut skipped_characters = 0usize;
    let mut uncovered_characters = 0usize;
    for (index, source) in prepared.iter().enumerate() {
        let length = source.text.chars().count();
        let mut windows = by_parent[index].clone();
        windows.sort_by_key(|item| item.get("char_start").and_then(Value::as_u64).unwrap_or(0));
        let mut cursor = 0usize;
        let mut covered = 0usize;
        if windows.is_empty() {
            return Err("invalid_output".to_string());
        }
        for item in windows {
            let start = read_usize(item, "char_start")?;
            let end = read_usize(item, "char_end")?;
            if start > end || end > length || start < cursor {
                return Err("invalid_output".to_string());
            }
            uncovered_characters += start - cursor;
            cursor = end;
            covered += end - start;
            let scored = item
                .get("scored")
                .and_then(Value::as_bool)
                .ok_or("invalid_output")?;
            if !scored {
                skipped_characters += end - start;
                chunks.push(AiChunk {
                    location: source.location.clone(),
                    language: source.language.clone(),
                    language_forced: source.forced,
                    char_start: start,
                    char_end: end,
                    token_count: item
                        .get("token_count")
                        .and_then(Value::as_u64)
                        .map(|n| n as usize),
                    scored: false,
                    reason: item
                        .get("reason")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| Some("not_scored".to_string())),
                    predicted_class: None,
                    human_softmax: None,
                    ai_softmax: None,
                    score_0_to_100: None,
                });
                continue;
            }
            let model_key = item
                .get("model_key")
                .and_then(Value::as_str)
                .ok_or("invalid_output")?;
            if model_key != source.language {
                return Err("invalid_output".to_string());
            }
            if !models.iter().any(|model| model.key == model_key) {
                return Err("invalid_output".to_string());
            }
            let token_count = read_usize(item, "token_count")?;
            if token_count == 0 || token_count > 512 {
                return Err("invalid_output".to_string());
            }
            let human = read_unit(item, "human_softmax")?;
            let ai = read_unit(item, "ai_softmax")?;
            if (human + ai - 1.0).abs() > 0.02 {
                return Err("invalid_output".to_string());
            }
            let predicted = read_usize(item, "predicted_index")?;
            let expected = if ai > human { 1 } else { 0 };
            if predicted != expected || predicted > 1 {
                return Err("invalid_output".to_string());
            }
            let class_name = if predicted == 1 { "ai" } else { "human" };
            let span = end - start;
            scored_characters += span;
            if ai > 0.5 {
                flagged_characters += span;
            }
            chunks.push(AiChunk {
                location: source.location.clone(),
                language: source.language.clone(),
                language_forced: source.forced,
                char_start: start,
                char_end: end,
                token_count: Some(token_count),
                scored: true,
                reason: None,
                predicted_class: Some(class_name.to_string()),
                human_softmax: Some(round4(human)),
                ai_softmax: Some(round4(ai)),
                score_0_to_100: Some(round4(ai * 100.0)),
            });
        }
        if cursor > length {
            return Err("invalid_output".to_string());
        }
        uncovered_characters += length - cursor;
        let _ = covered;
    }
    let share = if scored_characters == 0 {
        None
    } else {
        Some(round4(flagged_characters as f64 / scored_characters as f64))
    };
    let status = if chunks.iter().any(|chunk| chunk.scored) {
        "ok"
    } else {
        "unavailable"
    };
    Ok(AiClassifierReport {
        status: status.to_string(),
        reason: if status == "ok" {
            None
        } else {
            Some("nothing_scored".to_string())
        },
        label: AI_SCORE_LABEL,
        distinction: AI_SCORE_DISTINCTION,
        flagged_share_definition: AI_FLAGGED_SHARE,
        not_a_zero_result: AI_NOT_RUN,
        chunks,
        scored_characters,
        flagged_characters,
        skipped_characters,
        uncovered_characters,
        flagged_character_share: share,
        models,
        notes: vec![
            AI_SCORE_DISTINCTION.to_string(),
            AI_FLAGGED_SHARE.to_string(),
            "Chunk scores are not averaged into a document-level AI percentage.".to_string(),
            "Short, mixed-language, academic, and non-native English text can be classified the wrong way. No accuracy figure is claimed.".to_string(),
        ],
    })
}

fn parse_models(value: &Value) -> Result<Vec<AiModelInfo>, String> {
    let mut models = Vec::new();
    for key in ["en", "zh"] {
        let Some(item) = value.get(key) else {
            continue;
        };
        let repo = item.get("repo").and_then(Value::as_str).unwrap_or("");
        let revision = item.get("revision").and_then(Value::as_str).unwrap_or("");
        let format = item
            .get("weight_format")
            .and_then(Value::as_str)
            .unwrap_or("");
        let expected_repo = if key == "en" {
            "yuchuantian/AIGC_detector_env3"
        } else {
            "yuchuantian/AIGC_detector_zhv3"
        };
        let expected_revision = if key == "en" {
            EN_REVISION
        } else {
            ZH_REVISION
        };
        if repo != expected_repo || revision != expected_revision || format != "safetensors" {
            return Err("model_identity_mismatch".to_string());
        }
        models.push(AiModelInfo {
            key: key.to_string(),
            repo: repo.to_string(),
            revision: revision.to_string(),
            weight_format: format.to_string(),
        });
    }
    Ok(models)
}

fn read_usize(value: &Value, key: &str) -> Result<usize, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .map(|number| number as usize)
        .ok_or_else(|| "invalid_output".to_string())
}

fn read_unit(value: &Value, key: &str) -> Result<f64, String> {
    let number = value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| "invalid_output".to_string())?;
    if !(0.0..=1.0).contains(&number) || !number.is_finite() {
        return Err("invalid_output".to_string());
    }
    Ok(number)
}

pub(crate) fn default_script() -> PathBuf {
    PathBuf::from("classifier/infer.py")
}

pub(crate) fn default_models_dir() -> PathBuf {
    PathBuf::from(".course-integrity/models")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chunk() -> PreparedChunk {
        PreparedChunk {
            id: "p1".to_string(),
            location: "paragraph 1".to_string(),
            language: "en".to_string(),
            forced: false,
            text: "alpha beta".to_string(),
        }
    }

    fn ok_json(ai: f64, human: f64, predicted: u8, tokens: u64) -> String {
        format!(
            r#"{{"status":"ok","max_tokens":512,"label_map":{{"0":"human","1":"ai"}},"models":{{"en":{{"repo":"yuchuantian/AIGC_detector_env3","revision":"{EN_REVISION}","weight_format":"safetensors"}}}},"chunks":[{{"parent_id":"p1","char_start":0,"char_end":10,"token_count":{tokens},"scored":true,"model_key":"en","human_softmax":{human},"ai_softmax":{ai},"predicted_index":{predicted}}}]}}"#
        )
    }

    #[test]
    fn valid_score_is_scaled_and_not_a_document_average() {
        let prepared = vec![sample_chunk()];
        let report = interpret(&prepared, &ok_json(0.25, 0.75, 0, 12)).unwrap();
        assert_eq!(report.status, "ok");
        assert_eq!(report.chunks[0].score_0_to_100, Some(25.0));
        assert_eq!(report.chunks[0].predicted_class.as_deref(), Some("human"));
        assert_eq!(report.flagged_characters, 0);
        assert_eq!(report.scored_characters, 10);
        assert!(report.flagged_character_share == Some(0.0));
        assert!(report.distinction.contains("not the percentage"));
    }

    #[test]
    fn out_of_range_score_and_wrong_model_are_rejected() {
        let prepared = vec![sample_chunk()];
        assert!(interpret(&prepared, &ok_json(1.2, -0.2, 1, 12)).is_err());
        let mismatched = ok_json(0.8, 0.2, 1, 12).replace(EN_REVISION, "deadbeef");
        assert!(interpret(&prepared, &mismatched).is_err());
        assert!(interpret(&prepared, &ok_json(0.8, 0.2, 1, 600)).is_err());
        assert!(interpret(&prepared, &ok_json(0.2, 0.8, 1, 12)).is_err());
    }

    #[test]
    fn not_run_has_no_numeric_score() {
        let report = not_run("missing_model");
        assert!(report.chunks.is_empty());
        assert!(report.flagged_character_share.is_none());
        assert_eq!(report.scored_characters, 0);
        let rendered = serde_json::to_string(&report).unwrap();
        assert!(!rendered.contains("score_0_to_100\":0"));
        assert!(rendered.contains("missing_model"));
    }

    #[test]
    fn sidecar_rejects_a_score_outside_the_unit_interval() {
        let script = std::env::temp_dir().join(format!(
            "course-integrity-bad-sidecar-{}.py",
            std::process::id()
        ));
        let program = format!(
            "import json,sys\njson.dump({{'status':'ok','max_tokens':512,'label_map':{{'0':'human','1':'ai'}},'models':{{'en':{{'repo':'yuchuantian/AIGC_detector_env3','revision':'{EN_REVISION}','weight_format':'safetensors'}}}},'chunks':[{{'parent_id':'p1','char_start':0,'char_end':10,'token_count':12,'scored':True,'model_key':'en','human_softmax':0.1,'ai_softmax':9,'predicted_index':1}}]}}, sys.stdout)\n"
        );
        std::fs::write(&script, program).unwrap();
        let report = classify_text(
            "alpha beta",
            "utf-8",
            &["paragraph 1".to_string()],
            &ClassifierOptions {
                enabled: true,
                models_dir: PathBuf::from("missing-models"),
                script: script.clone(),
                python: "python3".to_string(),
                language: "auto".to_string(),
                timeout: Duration::from_secs(10),
            },
        );
        let _ = std::fs::remove_file(script);
        assert_ne!(report.status, "ok");
        assert!(report
            .chunks
            .iter()
            .all(|chunk| chunk.score_0_to_100.is_none()));
        assert!(report.flagged_character_share.is_none());
    }

    #[test]
    fn sidecar_timeout_is_not_a_zero_score() {
        let script = std::env::temp_dir().join(format!(
            "course-integrity-slow-sidecar-{}.py",
            std::process::id()
        ));
        std::fs::write(&script, "import time\ntime.sleep(30)\n").unwrap();
        let report = classify_text(
            "alpha beta",
            "utf-8",
            &["paragraph 1".to_string()],
            &ClassifierOptions {
                enabled: true,
                models_dir: PathBuf::from("missing-models"),
                script: script.clone(),
                python: "python3".to_string(),
                language: "auto".to_string(),
                timeout: Duration::from_millis(400),
            },
        );
        let _ = std::fs::remove_file(script);
        assert_eq!(report.status, "not_run");
        assert!(report.flagged_character_share.is_none());
        let python_ok = std::process::Command::new("python3")
            .arg("-c")
            .arg("import sys")
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if python_ok {
            assert_eq!(report.reason.as_deref(), Some("timeout"));
        }
    }

    #[test]
    fn mixed_language_is_separate_from_english() {
        assert_eq!(detect_language("The beaker cooled on the bench."), "en");
        assert_eq!(detect_language("温度下降较快，随后变慢。"), "zh");
        assert_eq!(detect_language("The beaker 温度下降较快"), "mixed");
    }
}
