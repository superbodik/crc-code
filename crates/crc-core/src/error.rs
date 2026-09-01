use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("path `{0}` escapes the workspace root")]
    EscapesWorkspace(PathBuf),

    #[error("path `{0}` does not exist")]
    NotFound(PathBuf),

    #[error("`{path}` is {size} bytes, over the {limit} byte limit")]
    TooLarge {
        path: PathBuf,
        size: u64,
        limit: u64,
    },

    #[error("`{0}` is not valid UTF-8 text")]
    NotUtf8(PathBuf),

    #[error("`{path}` is at version {actual}, but the write expected {expected}")]
    VersionConflict {
        path: PathBuf,
        expected: u64,
        actual: u64,
    },

    #[error("io error on `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid search pattern")]
    Pattern(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("file watcher failed")]
    Watch(#[from] notify::Error),

    #[error("background task failed")]
    Join(#[from] tokio::task::JoinError),
}

impl CoreError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        let path = path.into();
        if source.kind() == std::io::ErrorKind::NotFound {
            CoreError::NotFound(path)
        } else {
            CoreError::Io { path, source }
        }
    }
}
