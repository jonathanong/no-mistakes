use super::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SqlOptions {
    sql_include: Vec<String>,
    #[serde(default)]
    sql_include_locked: bool,
}

pub(super) fn sql_patterns(config: &NoMistakesConfig, rule_ids: &[&str]) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    for id in rule_ids {
        for rule in config.rule_applications(id) {
            let options: SqlOptions = rule.try_rule_options()?;
            patterns.extend(crate::codebase::postgres::profile_sql_include(
                id,
                options.sql_include,
                options.sql_include_locked,
            ));
        }
    }
    patterns.sort();
    patterns.dedup();
    Ok(patterns)
}
