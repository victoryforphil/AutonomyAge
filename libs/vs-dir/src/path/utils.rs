use std::path::{Path, PathBuf};

pub(crate) fn absolute_start(start: &Path, cwd: &Path) -> PathBuf {
    let path = absolute_path(start.to_path_buf(), cwd);
    if path.is_file() {
        path.parent().unwrap_or(path.as_path()).to_path_buf()
    } else {
        path
    }
}

pub(crate) fn absolute_path(path: PathBuf, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}
