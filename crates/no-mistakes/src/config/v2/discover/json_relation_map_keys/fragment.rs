use super::{Rule, RULE_ID};
use anyhow::Result;
use std::path::Path;

pub(super) fn validate(source: &str, path: &Path, section: &str) -> Result<()> {
    if section != "rules" {
        return Ok(());
    }
    let duplicate = match path.extension().and_then(|extension| extension.to_str()) {
        Some("yaml" | "yml") => serde_yaml::from_str::<Vec<Rule>>(source)
            .ok()
            .and_then(super::yaml_rules_duplicate_at),
        Some("json" | "jsonc") => super::json_duplicate_input(
            source,
            path.extension()
                .is_some_and(|extension| extension == "jsonc"),
            true,
        ),
        _ => None,
    };
    if let Some((index, key)) = duplicate {
        let origin = crate::config::source_locations::ConfigPath::default()
            .index(index)
            .key("options")
            .key("keyColumnMap")
            .key(&key);
        let line = crate::config::source_locations::ConfigLocations::parse(source, path)
            .and_then(|locations| locations.line(&origin))
            .unwrap_or(1);
        anyhow::bail!(
            "{}:{line}: {RULE_ID} option keyColumnMap: duplicate key {key}",
            path.display()
        );
    }
    Ok(())
}
