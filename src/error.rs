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
    Extract {
        path: PathBuf,
        message: String,
    },
    Output(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::BadThreshold(_) | Error::Output(_) => 2,
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
                "{}: not valid UTF-8. .txt and .md must be UTF-8. For Word, use .docx rather than renaming a binary file.",
                path.display()
            ),
            Error::TooLarge { path, bytes } => write!(
                f,
                "{}: file is {bytes} bytes, over the {} byte limit. Split the draft or export a smaller PDF.",
                path.display(),
                crate::method::MAX_FILE_BYTES
            ),
            Error::Unsupported { path } => write!(
                f,
                "{}: unsupported type. This version reads UTF-8 .txt, .md, .docx, and text-based .pdf. Legacy .doc is not read.",
                path.display()
            ),
            Error::NoInputs { path, detail } => {
                if detail.is_empty() {
                    write!(
                        f,
                        "{}: no .txt, .md, .docx, or .pdf files found",
                        path.display()
                    )
                } else {
                    write!(
                        f,
                        "{}: no readable .txt, .md, .docx, or .pdf files found. {detail}",
                        path.display()
                    )
                }
            }
            Error::NotFound { path } => write!(
                f,
                "{}: not found. Pass a .txt, .md, .docx, or .pdf file, or a folder that contains them.",
                path.display()
            ),
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
            Error::Extract { path, message } => write!(f, "{}: {message}", path.display()),
            Error::Output(message) => write!(f, "{message}"),
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
