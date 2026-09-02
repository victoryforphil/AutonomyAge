# vs-dir

Small directory locator for the Victory Suite workspace.

`DirLocator` keeps one resolution path for an application and its subdirectories:

```text
configured ENV base/.dot
→ cwd/.dot
→ Git root/.dot
→ Cargo crate root/.dot
→ HOME/.dot
```

The first existing project directory wins. The fallback home directory is
created on demand. `logs_dir()` and `data_dir()` are created below the same
selected application directory.

## Usage

```rust
use std::path::Path;
use anyhow::Result;
use vs_dir::{DirConfig, DirLocator};

fn main() -> Result<()> {
    let config = DirConfig::new(".vs").with_env("VS_DIR_BASE");
    let locator = DirLocator::new(config, Path::new("."))?;

    println!("app={}", locator.app_dir()?.display());
    println!("logs={}", locator.logs_dir()?.display());
    println!("data={}", locator.data_dir()?.display());
    Ok(())
}
```

Environment values are explicit base directories; the dot component is always
appended. Empty values are ignored. Relative values resolve against the launch
cwd.

The crate deliberately exposes a small struct-based API. Configuration parsing,
logging setup, and artifact lifecycle remain consumer responsibilities.
