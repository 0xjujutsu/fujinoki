use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

mod js_bindings;
pub mod typegen;

use self::js_bindings::create_cjs_binding;

pub fn write_js_binding(
    idents: &[String],
    local_name: &str,
    package_name: &str,
    output: PathBuf,
) -> Result<()> {
    if idents.is_empty() {
        return Ok(());
    }

    let cjs = create_cjs_binding(local_name, package_name, idents);

    println!("Writing js binding to:");
    println!("  {:?}", output);

    fs::write(&output, cjs)
        .with_context(|| format!("Failed to write js binding file to {:?}", output))?;

    Ok(())
}
