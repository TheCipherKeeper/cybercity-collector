use ccc_core::Policy;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum HostError {
    #[error("path not allowed: {0}")]
    NotAllowed(PathBuf),
    #[error("path traversal detected: {0}")]
    Traversal(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct HostBridge {
    policy: Policy,
}

impl HostBridge {
    pub fn new(policy: Policy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// Read a file from the host if the path is allowed by policy.
    /// Path traversal outside of allowed prefixes is rejected.
    pub async fn read_file(&self, path: &Path) -> Result<Vec<u8>, HostError> {
        let canonical = normalize(path).await?;
        if !self.policy.can_read_file(&canonical) {
            warn!("rejecting read of disallowed path: {:?}", canonical);
            return Err(HostError::NotAllowed(canonical));
        }
        debug!("reading allowed file: {:?}", canonical);
        tokio::fs::read(&canonical).await.map_err(HostError::Io)
    }

    /// List entries inside an allowed directory (non-recursive).
    pub async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, HostError> {
        let canonical = normalize(path).await?;
        if !self.policy.can_read_file(&canonical) {
            return Err(HostError::NotAllowed(canonical));
        }
        let mut entries = vec![];
        let mut dir = tokio::fs::read_dir(&canonical).await?;
        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry.path());
        }
        Ok(entries)
    }
}

async fn normalize(path: &Path) -> Result<PathBuf, HostError> {
    // Reject any component that goes upward: no escaping allowed roots.
    for comp in path.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return Err(HostError::Traversal(path.to_path_buf()));
        }
    }
    // Use absolute path if it already is; otherwise anchor at /.
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new("/").join(path)
    };
    // Resolve symlinks and ".." safely.
    let canonical = tokio::fs::canonicalize(&abs)
        .await
        .unwrap_or_else(|_| abs.clean());
    Ok(canonical)
}

trait PathClean {
    fn clean(&self) -> PathBuf;
}

impl PathClean for PathBuf {
    fn clean(&self) -> PathBuf {
        let mut out = PathBuf::from("/");
        for comp in self.components() {
            match comp {
                std::path::Component::ParentDir => {
                    let _ = out.pop();
                }
                std::path::Component::CurDir => {}
                std::path::Component::RootDir => {}
                other => out.push(other.as_os_str()),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_traversal() {
        let policy = Policy::default();
        let bridge = HostBridge::new(policy);
        let res = bridge.read_file(Path::new("/var/log/../etc/passwd")).await;
        assert!(matches!(res, Err(HostError::Traversal(_))));
    }
}
