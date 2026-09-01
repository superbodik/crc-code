use std::path::{Component, Path, PathBuf};

use crate::error::{CoreError, Result};

/// The opened project root.
///
/// Every path that reaches the engine is resolved through [`Workspace::resolve`],
/// including paths written by an AI agent. Nothing outside the root can be read
/// or written, so a hallucinated `../../.ssh/id_rsa` fails closed instead of
/// reaching the disk.
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

    /// Turn a workspace-relative (or in-workspace absolute) path into an
    /// absolute one, rejecting anything that climbs out of the root.
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

        // A symlink that lives inside the root can still point outside it, so
        // check the real target whenever the path already exists.
        match dunce::canonicalize(&lexical) {
            Ok(real) if real.starts_with(&self.root) => Ok(real),
            Ok(_) => Err(CoreError::EscapesWorkspace(path.to_path_buf())),
            // Not on disk yet: a create/write to a lexically safe path.
            Err(_) => Ok(lexical),
        }
    }

    /// The path as the UI and the agents should see it: relative to the root.
    pub fn relative<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }
}

/// Resolve `.` and `..` without touching the disk.
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
