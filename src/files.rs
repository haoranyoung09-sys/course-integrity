use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::method::MAX_FILE_BYTES;
use crate::tokenize::canonical_label;

pub(crate) struct LoadedFile {
    pub label: String,
    pub text: String,
    pub bytes: u64,
}

pub(crate) struct LoadedSet {
    pub files: Vec<LoadedFile>,
    pub warnings: Vec<String>,
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
    let mut loaded = read_utf8(path)?;
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
        match read_utf8(&file) {
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
            if is_supported(&path) {
                paths.push(path);
            } else if is_deferred_format(&path) {
                warnings.push(format!(
                    "{}: DOCX and PDF are not read in this version; skipped",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

fn read_utf8(path: &Path) -> Result<LoadedFile, Error> {
    let meta = fs::metadata(path).map_err(|source| io_at(path, source))?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(Error::TooLarge {
            path: path.to_path_buf(),
            bytes: meta.len(),
        });
    }
    let bytes = fs::read(path).map_err(|source| io_at(path, source))?;
    let text = String::from_utf8(bytes).map_err(|_| Error::NotUtf8 {
        path: path.to_path_buf(),
    })?;
    Ok(LoadedFile {
        label: canonical_label(&path.to_string_lossy()),
        text,
        bytes: meta.len(),
    })
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

fn is_supported(path: &Path) -> bool {
    extension_is(path, &["txt", "md"])
}

fn is_deferred_format(path: &Path) -> bool {
    extension_is(path, &["pdf", "docx", "doc"])
}

fn extension_is(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .is_some_and(|ext| extensions.iter().any(|candidate| ext == *candidate))
}
