use super::*;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SqlOptions {
    sql_include: Vec<String>,
}

pub(super) fn sql_patterns(config: &NoMistakesConfig, rule_ids: &[&str]) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    for id in rule_ids {
        for rule in config.rule_applications(id) {
            let options: SqlOptions = rule.try_rule_options()?;
            patterns.extend(
                if options.sql_include.is_empty()
                    && !matches!(
                        *id,
                        "postgres-explicit-columns"
                            | "postgres-bounded-statements"
                            | "postgres-no-offset"
                    )
                {
                    super::super::PostgresSchemaOptions::default().sql_include
                } else {
                    options.sql_include
                },
            );
        }
    }
    patterns.sort();
    patterns.dedup();
    Ok(patterns)
}
