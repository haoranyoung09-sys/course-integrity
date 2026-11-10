use std::fmt;
use std::path::PathBuf;

/// Recoverable failures for the library and the CLI.
#[derive(Debug)]
pub enum Error {
    Io {
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    NotUtf8 {
        path: PathBuf,
    },
    TooLarge {
        path: PathBuf,
        bytes: u64,
    },
    Unsupported {
        path: PathBuf,
    },
    NoInputs {
        path: PathBuf,
        detail: String,
    },
    NotFound {
        path: PathBuf,
    },
    Database {
        path: PathBuf,
        message: String,
    },
    BadThreshold(f64),
    LabelRequiresFile,
    Schema {
        path: PathBuf,
        found: String,
    },
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::BadThreshold(_) => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io {
                path: Some(path),
                source,
            } => write!(f, "{}: {source}", path.display()),
            Error::Io { path: None, source } => write!(f, "{source}"),
            Error::NotUtf8 { path } => write!(
                f,
                "{}: not valid UTF-8; this version reads only UTF-8 .txt and .md files",
                path.display()
            ),
            Error::TooLarge { path, bytes } => write!(
                f,
                "{}: file is {bytes} bytes, over the {} byte limit for this version",
                path.display(),
                crate::method::MAX_FILE_BYTES
            ),
            Error::Unsupported { path } => write!(
                f,
                "{}: only UTF-8 .txt and .md files are supported in this version (DOCX and PDF are not read)",
                path.display()
            ),
            Error::NoInputs { path, detail } => {
                if detail.is_empty() {
                    write!(f, "{}: no .txt or .md files found", path.display())
                } else {
                    write!(
                        f,
                        "{}: no .txt or .md files found. {detail}",
                        path.display()
                    )
                }
            }
            Error::NotFound { path } => write!(f, "{}: not found", path.display()),
            Error::Database { path, message } => {
                write!(f, "{}: {message}", path.display())
            }
            Error::BadThreshold(value) => write!(
                f,
                "min-containment must be greater than 0 and at most 1, got {value}"
            ),
            Error::LabelRequiresFile => {
                write!(
                    f,
                    "--label can be used only with a single file, not a directory"
                )
            }
            Error::Schema { path, found } => write!(
                f,
                "{}: database schema version is {found}; this build only opens schema {}. Choose a different --db path",
                path.display(),
                crate::method::DB_SCHEMA_VERSION
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
