use super::snapshot::Snapshot;
use anyhow::{bail, Result};

const HINT: &str = "generate the catalog with `no-mistakes postgres catalog`";

/// Read the single catalog format that `no-mistakes postgres catalog` writes.
pub(super) fn parse_snapshot(path: &str, value: serde_json::Value) -> Result<Snapshot> {
    if value
        .get("formatVersion")
        .and_then(serde_json::Value::as_u64)
        != Some(2)
    {
        bail!(
            "schemaCatalogPath {path} must be a no-mistakes schema catalog with formatVersion 2; {HINT}"
        );
    }
    serde_path_to_error::deserialize(value).map_err(|error| {
        let field = error.path().to_string();
        let message = error.inner();
        if field == "." {
            anyhow::anyhow!("schemaCatalogPath {path} has an invalid schema: {message}; {HINT}")
        } else {
            anyhow::anyhow!(
                "schemaCatalogPath {path} has an invalid schema: {field}: {message}; {HINT}"
            )
        }
    })
}
