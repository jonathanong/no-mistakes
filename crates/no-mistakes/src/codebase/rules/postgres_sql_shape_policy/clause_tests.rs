use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

fn config(entries: &str) -> NoMistakesConfig {
    NoMistakesConfig { rules: vec![RuleDef { rule: RULE_ID.into(), scope: Some(RuleScope::Repository), options: serde_yaml::from_str(&format!("executorNames: []\nbannedShapes: [banned-function-call]\nshapeOptions:\n  bannedFunctionCall:\n    functions: {entries}")).unwrap(), ..Default::default() }], ..Default::default() }
}

#[test]
fn every_scoped_configuration_error_is_rejected() {
    for entries in [
        "[{}]",
        "[{name: ' '}]",
        "[{name: uuidv7, clauses: []}]",
        "[{name: uuidv7, clauses: [WHERE]}]",
        "[{name: uuidv7, clauses: [where, where]}]",
        "[{name: uuidv7, hint: ' '}]",
        "[uuidv7, {name: UUIDV7}]",
        "[{name: uuidv7}, {name: uuidv7}]",
    ] {
        let cfg = config(entries);
        let options: Options = cfg.rules[0].try_rule_options().unwrap();
        let error = compile_options(&options).err().expect(entries).to_string();
        assert!(
            error.starts_with(
                "postgres-sql-shape-policy option shapeOptions.bannedFunctionCall.functions"
            ),
            "{error}"
        );
    }
}
