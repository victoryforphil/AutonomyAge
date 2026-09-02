use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Explicit inputs for one directory locator.
#[derive(Clone, Copy, Debug)]
pub struct DirConfig<'a> {
    /// Literal directory component, including the leading dot.
    pub dot: &'a str,
    /// Optional environment variable naming the override base directory.
    pub env_var: Option<&'a str>,
}

impl<'a> DirConfig<'a> {
    /// Build a configuration with no environment override.
    pub fn new(dot: &'a str) -> Self {
        Self { dot, env_var: None }
    }

    /// Set the environment variable naming the override base directory.
    pub fn with_env(mut self, variable: &'a str) -> Self {
        self.env_var = Some(variable);
        self
    }

    pub(crate) fn env_base(&self, cwd: &Path) -> Option<PathBuf> {
        let variable = self.env_var?;
        let value = env::var_os(variable)?;
        if value.is_empty() {
            return None;
        }
        Some(crate::path::utils::absolute_path(PathBuf::from(value), cwd))
    }

    pub(crate) fn dot_path(&self) -> Result<PathBuf> {
        let path = Path::new(self.dot);
        if self.dot.is_empty() || path.is_absolute() {
            return Err(anyhow!("dot directory must be a non-empty relative path"));
        }
        Ok(path.to_path_buf())
    }
}
