use super::{generate, PostgresCatalogOptions};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;

/// Generate a catalog, atomically replace the output, and return the catalog.
pub fn generate_to(options: &PostgresCatalogOptions, output: &Path) -> Result<serde_json::Value> {
    anyhow::ensure!(
        !output.as_os_str().is_empty(),
        "output must be a non-empty path"
    );
    let catalog = generate(options)?;
    publish_catalog(output, |file| writeln!(file, "{catalog:#}"))
        .with_context(|| format!("failed to write catalog {}", output.display()))?;
    Ok(catalog)
}

fn publish_catalog(
    output: &Path,
    write: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>,
) -> Result<()> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    write(temporary.as_file_mut())?;
    temporary.persist(output)?;
    Ok(())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
