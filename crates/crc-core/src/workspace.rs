use std::path::{Component, Path, PathBuf};

use crate::error::{CoreError, Result};

#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let raw = root.as_ref();
        let root = dunce::canonicalize(raw).map_err(|e| CoreError::io(raw, e))?;
        if !root.is_dir() {
            return Err(CoreError::NotFound(root));
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = path.as_ref();
        let joined = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        let lexical = normalize(&joined);
        if !lexical.starts_with(&self.root) {
            return Err(CoreError::EscapesWorkspace(path.to_path_buf()));
        }

        match dunce::canonicalize(&lexical) {
            Ok(real) if real.starts_with(&self.root) => Ok(real),
            Ok(_) => Err(CoreError::EscapesWorkspace(path.to_path_buf())),
            Err(_) => Ok(lexical),
        }
    }

    pub fn relative<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
