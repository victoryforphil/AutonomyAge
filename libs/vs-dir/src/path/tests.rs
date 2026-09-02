use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{DirConfig, DirLocator};

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static NEXT: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

fn env_lock() -> &'static Mutex<()> {
    &ENV_LOCK
}

fn temp_dir(name: &str) -> PathBuf {
    let mut counter = NEXT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    *counter += 1;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("vs-dir-{name}-{nanos}-{}", *counter));
    fs::create_dir_all(&path).unwrap();
    path
}

fn locator<'a>(dot: &'a str, env_var: Option<&'a str>, cwd: &Path) -> DirLocator<'a> {
    let config = env_var
        .map(|variable| DirConfig::new(dot).with_env(variable))
        .unwrap_or_else(|| DirConfig::new(dot));
    DirLocator::new(config, cwd).unwrap()
}

fn restore_env(name: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(value) => env::set_var(name, value),
        None => env::remove_var(name),
    }
}

#[test]
fn resolution_order_scenario() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let home = temp_dir("home");
    let repository = temp_dir("repository");
    let crate_root = repository.join("crate");
    let cwd = crate_root.join("src").join("nested");
    fs::create_dir_all(&cwd).unwrap();
    fs::create_dir(repository.join(".git")).unwrap();
    fs::write(
        crate_root.join("Cargo.toml"),
        "[package]\nname = \"crate\"\n",
    )
    .unwrap();
    let previous_home = env::var_os("HOME");
    env::set_var("HOME", &home);
    let locator = locator(".lil", None, &cwd);

    fs::create_dir(cwd.join(".lil")).unwrap();
    assert_eq!(locator.app_dir().unwrap(), cwd.join(".lil"));

    fs::remove_dir(cwd.join(".lil")).unwrap();
    fs::create_dir(repository.join(".lil")).unwrap();
    assert_eq!(locator.app_dir().unwrap(), repository.join(".lil"));

    fs::remove_dir(repository.join(".lil")).unwrap();
    fs::create_dir(crate_root.join(".lil")).unwrap();
    assert_eq!(locator.app_dir().unwrap(), crate_root.join(".lil"));

    fs::remove_dir(crate_root.join(".lil")).unwrap();
    assert_eq!(locator.app_dir().unwrap(), home.join(".lil"));

    restore_env("HOME", previous_home);
    fs::remove_dir_all(home).unwrap();
    fs::remove_dir_all(repository).unwrap();
}

#[test]
fn environment_override_scenario() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let home = temp_dir("empty-home");
    let override_base = temp_dir("override");
    let variable = "VS_DIR_TEST_BASE";
    let previous_home = env::var_os("HOME");
    env::set_var("HOME", &home);
    env::set_var(variable, &override_base);
    let locator = locator(".lil", Some(variable), Path::new("."));

    assert_eq!(locator.app_dir().unwrap(), override_base.join(".lil"));

    env::set_var(variable, "");
    assert_eq!(locator.app_dir().unwrap(), home.join(".lil"));

    env::remove_var(variable);
    restore_env("HOME", previous_home);
    fs::remove_dir_all(home).unwrap();
    fs::remove_dir_all(override_base).unwrap();
}

#[test]
fn subdirectory_scenario() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base = temp_dir("subdirectories");
    let cwd = base.join("project");
    fs::create_dir_all(cwd.join(".lil")).unwrap();
    let locator = locator(".lil", None, &cwd);

    let logs = locator.logs_dir().unwrap();
    let data = locator.data_dir().unwrap();

    assert_eq!(logs, cwd.join(".lil/logs"));
    assert_eq!(data, cwd.join(".lil/data"));
    assert!(logs.is_dir());
    assert!(data.is_dir());
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn ensure_directory_scenario() {
    let root = temp_dir("ensure");
    let target = root.join("one").join("two");
    let locator = locator(".lil", None, Path::new("."));

    assert_eq!(locator.ensure_dir(&target).unwrap(), target);
    assert!(target.is_dir());
    fs::remove_dir_all(root).unwrap();
}
