use super::*;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct WriteOptions {
    include: Vec<String>,
}

pub(in super::super) fn write_patterns(config: &NoMistakesConfig) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    for rule in config.rule_applications("postgres-no-generated-column-writes") {
        let options: WriteOptions = rule.try_rule_options()?;
        patterns.extend(if options.include.is_empty() {
            vec!["**/*.sql".into()]
        } else {
            options.include
        });
    }
    Ok(patterns)
}
