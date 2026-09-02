use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::utils::{absolute_path, absolute_start};
use crate::config::DirConfig;

/// Owns directory resolution for one configured application and launch cwd.
#[derive(Clone, Debug)]
pub struct DirLocator<'a> {
    config: DirConfig<'a>,
    cwd: PathBuf,
}

impl<'a> DirLocator<'a> {
    /// Create a locator rooted at `cwd`.
    pub fn new(config: DirConfig<'a>, cwd: &Path) -> Result<Self> {
        let launch_cwd = env::current_dir().context("resolving current directory")?;
        Ok(Self {
            config,
            cwd: absolute_path(cwd.to_path_buf(), &launch_cwd),
        })
    }

    /// Resolve the user home directory using HOME, then USERPROFILE.
    pub fn home_dir() -> Result<PathBuf> {
        ["HOME", "USERPROFILE"]
            .into_iter()
            .filter_map(env::var_os)
            .find(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("neither HOME nor USERPROFILE is set"))
    }

    /// Resolve and create the app directory using this order:
    /// configured env base, cwd, Git root, Cargo crate root, then home.
    pub fn app_dir(&self) -> Result<PathBuf> {
        let dot = self.config.dot_path()?;
        if let Some(base) = self.config.env_base(&self.cwd) {
            return self.ensure_dir(&base.join(&dot));
        }

        for base in self.project_bases() {
            let path = base.join(&dot);
            if path.is_dir() {
                return Ok(path);
            }
        }

        self.ensure_dir(&Self::home_dir()?.join(dot))
    }

    /// Resolve and create the app logs directory.
    pub fn logs_dir(&self) -> Result<PathBuf> {
        self.ensure_dir(&self.app_dir()?.join("logs"))
    }

    /// Resolve and create the app data directory.
    pub fn data_dir(&self) -> Result<PathBuf> {
        self.ensure_dir(&self.app_dir()?.join("data"))
    }

    /// Create one selected directory and return it.
    pub fn ensure_dir(&self, path: &Path) -> Result<PathBuf> {
        fs::create_dir_all(path)
            .with_context(|| format!("creating directory {}", path.display()))?;
        Ok(path.to_path_buf())
    }

    fn project_bases(&self) -> Vec<PathBuf> {
        let cwd = absolute_start(&self.cwd, &self.cwd);
        let mut bases = vec![cwd.clone()];
        append_unique(&mut bases, git_root_from(&cwd));
        append_unique(&mut bases, cargo_crate_root_from(&cwd));
        bases
    }
}

fn git_root_from(start: &Path) -> Option<PathBuf> {
    nearest_ancestor(start, |path| {
        path.join(".git").is_dir() || path.join(".git").is_file()
    })
}

fn cargo_crate_root_from(start: &Path) -> Option<PathBuf> {
    nearest_ancestor(start, |path| {
        fs::read_to_string(path.join("Cargo.toml"))
            .map(|manifest| manifest.lines().any(|line| line.trim() == "[package]"))
            .unwrap_or(false)
    })
}

fn nearest_ancestor<F>(start: &Path, matches: F) -> Option<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let mut current = Some(start);
    while let Some(path) = current {
        if matches(path) {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }
    None
}

fn append_unique(paths: &mut Vec<PathBuf>, path: Option<PathBuf>) {
    if let Some(path) = path {
        if !paths.iter().any(|candidate| candidate == &path) {
            paths.push(path);
        }
    }
}
