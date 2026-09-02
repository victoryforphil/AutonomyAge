use std::path::Path;

use anyhow::Result;
use vs_dir::{DirConfig, DirLocator};

fn main() -> Result<()> {
    let config = DirConfig::new(".vs").with_env("VS_DIR_EXAMPLE_BASE");
    let locator = DirLocator::new(config, Path::new("."))?;

    let app = locator.app_dir()?;
    let logs = locator.logs_dir()?;
    let data = locator.data_dir()?;
    println!("app={}", app.display());
    println!("logs={}", logs.display());
    println!("data={}", data.display());
    Ok(())
}
