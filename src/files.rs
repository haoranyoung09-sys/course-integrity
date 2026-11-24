use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::extract::{extract_docx, extract_pdf};
use crate::method::{EXTRACTION_DESC, MAX_FILE_BYTES};
use crate::tokenize::{canonical_label, paragraphs};

pub(crate) struct LoadedFile {
    pub label: String,
    pub text: String,
    pub bytes: u64,
    pub origin: String,
    pub locations: Vec<String>,
    pub diagnostics: Vec<String>,
}

pub(crate) struct LoadedSet {
    pub files: Vec<LoadedFile>,
    pub warnings: Vec<String>,
}

pub(crate) fn load_many_inputs(
    paths: &[PathBuf],
    label_override: Option<&str>,
) -> Result<LoadedSet, Error> {
    if paths.len() == 1 {
        return load_inputs(&paths[0], label_override);
    }
    if label_override.is_some() {
        return Err(Error::LabelRequiresFile);
    }
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_labels = BTreeSet::new();
    for path in paths {
        let loaded = load_inputs(path, None)?;
        warnings.extend(loaded.warnings);
        for file in loaded.files {
            if seen_labels.insert(file.label.clone()) {
                files.push(file);
            }
        }
    }
    Ok(LoadedSet { files, warnings })
}

pub(crate) fn load_inputs(path: &Path, label_override: Option<&str>) -> Result<LoadedSet, Error> {
    if !path.exists() {
        return Err(Error::NotFound {
            path: path.to_path_buf(),
        });
    }
    if path.is_dir() {
        if label_override.is_some() {
            return Err(Error::LabelRequiresFile);
        }
        return load_directory(path);
    }
    if !path.is_file() {
        return Err(Error::Unsupported {
            path: path.to_path_buf(),
        });
    }
    if !is_supported(path) {
        return Err(Error::Unsupported {
            path: path.to_path_buf(),
        });
    }
    let mut loaded = load_file(path)?;
    if let Some(label) = label_override {
        loaded.label = canonical_label(label);
    }
    Ok(LoadedSet {
        files: vec![loaded],
        warnings: Vec::new(),
    })
}

fn load_directory(path: &Path) -> Result<LoadedSet, Error> {
    let mut paths = Vec::new();
    let mut warnings = Vec::new();
    walk(path, &mut paths, &mut warnings)?;
    let mut files = Vec::new();
    for file in paths {
        match load_file(&file) {
            Ok(loaded) => files.push(loaded),
            Err(error) => warnings.push(error.to_string()),
        }
    }
    files.sort_by(|left, right| left.label.cmp(&right.label));
    if files.is_empty() {
        let detail = warnings.join(" ");
        return Err(Error::NoInputs {
            path: path.to_path_buf(),
            detail,
        });
    }
    Ok(LoadedSet { files, warnings })
}

fn walk(
    dir: &Path,
    paths: &mut Vec<std::path::PathBuf>,
    warnings: &mut Vec<String>,
) -> Result<(), Error> {
    let entries = fs::read_dir(dir).map_err(|source| Error::Io {
        path: Some(dir.to_path_buf()),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io {
            path: Some(dir.to_path_buf()),
            source,
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        let file_type = entry.file_type().map_err(|source| Error::Io {
            path: Some(entry.path()),
            source,
        })?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            if name == "reports" || name == "data" {
                continue;
            }
            walk(&path, paths, warnings)?;
        } else if file_type.is_file() {
            if is_generated_review(&path) {
                continue;
            }
            if is_supported(&path) {
                paths.push(path);
            } else if is_legacy_doc(&path) {
                warnings.push(format!(
                    "{}: legacy Word .doc is not read. Save as .docx or .pdf.",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

fn load_file(path: &Path) -> Result<LoadedFile, Error> {
    let bytes = read_limited(path)?;
    let extension = extension_of(path);
    let loaded = if extension == "docx" {
        let extracted = extract_docx(path, &bytes)?;
        LoadedFile {
            label: canonical_label(&path.to_string_lossy()),
            text: extracted.text,
            bytes: byte_len(&bytes),
            origin: extracted.origin.to_string(),
            locations: extracted.locations,
            diagnostics: extracted.diagnostics,
        }
    } else if extension == "pdf" {
        let extracted = extract_pdf(path, &bytes)?;
        LoadedFile {
            label: canonical_label(&path.to_string_lossy()),
            text: extracted.text,
            bytes: byte_len(&bytes),
            origin: extracted.origin.to_string(),
            locations: extracted.locations,
            diagnostics: extracted.diagnostics,
        }
    } else {
        let size = byte_len(&bytes);
        let text = String::from_utf8(bytes).map_err(|_| Error::NotUtf8 {
            path: path.to_path_buf(),
        })?;
        let locations = paragraphs(&text)
            .into_iter()
            .enumerate()
            .map(|(index, _)| format!("paragraph {}", index + 1))
            .collect();
        LoadedFile {
            label: canonical_label(&path.to_string_lossy()),
            text,
            bytes: size,
            origin: "utf-8".to_string(),
            locations,
            diagnostics: vec![
                "Read as UTF-8 text. Paragraph locations are blank-line blocks in the file."
                    .to_string(),
                EXTRACTION_DESC.to_string(),
            ],
        }
    };
    Ok(loaded)
}

fn read_limited(path: &Path) -> Result<Vec<u8>, Error> {
    let file = fs::File::open(path).map_err(|source| io_at(path, source))?;
    if let Ok(meta) = file.metadata() {
        if meta.len() > MAX_FILE_BYTES {
            return Err(Error::TooLarge {
                path: path.to_path_buf(),
                bytes: meta.len(),
            });
        }
    }
    let mut limited = file.take(MAX_FILE_BYTES + 1);
    let mut bytes = Vec::new();
    limited
        .read_to_end(&mut bytes)
        .map_err(|source| io_at(path, source))?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(Error::TooLarge {
            path: path.to_path_buf(),
            bytes: bytes.len() as u64,
        });
    }
    Ok(bytes)
}

fn io_at(path: &Path, source: std::io::Error) -> Error {
    if source.kind() == std::io::ErrorKind::NotFound {
        Error::NotFound {
            path: path.to_path_buf(),
        }
    } else {
        Error::Io {
            path: Some(path.to_path_buf()),
            source,
        }
    }
}

fn is_generated_review(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase().ends_with(".review.md"))
        .unwrap_or(false)
}

fn is_supported(path: &Path) -> bool {
    matches!(extension_of(path).as_str(), "txt" | "md" | "docx" | "pdf")
}

fn is_legacy_doc(path: &Path) -> bool {
    extension_of(path) == "doc"
}

fn byte_len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

fn extension_of(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default()
}
