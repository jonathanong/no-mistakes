use super::super::compile::compile;
use super::super::scan::scan;
use super::super::Options;
use crate::codebase::postgres::SchemaCatalog;
use crate::codebase::rules::RuleFinding;

pub(super) const INDEX: &str = "\
schemaCatalogPath: schema.json\n\
patterns:\n  index: '^idx_{table}__[a-z0-9_]+$'\n\
abbreviations:\n  enabled: true\n";

pub(super) const PROPOSED: &str = r#"
schemaCatalogPath: db/schema.json
patterns:
  index: '^idx_{table}__[a-z0-9_]+$'
  uniqueIndex: '^(idx|uq)_{table}__[a-z0-9_]+$'
  trigger: '^trigger_[a-z0-9_]+$'
  function: '^fn_[a-z0-9_]+$'
  triggerFunction: '^fn_(reject|update|project|create|lock)_[a-z0-9_]+$'
  view: '^view_[a-z0-9_]+$'
  materializedView: '^mv_[a-z0-9_]+$'
  table: '^[a-z][a-z0-9_]*$'
  column: '^[a-z][a-z0-9_]*$'
  enum: '^[a-z][a-z0-9_]*$'
checkConstraintBackedIndexes: false
tableMinWords: 2
abbreviations:
  enabled: true
  minLetters: 3
plural:
  enabled: true
  objects: [table, enum]
  irregularPlurals: { person: people, child: children }
  uncountable: [data, metadata, feedback, media]
  nonPluralTokens: [status, analysis, sms, news, series]
  ignorePatterns: ['^link__']
deniedTokens:
  - token: cfg
    replacement: configuration
  - token: tmp
    replacement: temporary
spelling:
  acknowledgement: acknowledgment
doubleUnderscore:
  allowPattern: '^link__[a-z0-9]+(_[a-z0-9]+)*__[a-z0-9_]+__[a-z0-9_]+$'
allow:
  - object: 'table:legacy_cfg_values'
    reason: 'Name owned by an external replication tool'
"#;

pub(super) fn messages(yaml: &str, body: serde_json::Value) -> Vec<String> {
    findings(yaml, body)
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

pub(super) fn findings(yaml: &str, body: serde_json::Value) -> Vec<RuleFinding> {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let compiled = compile(&options, None).unwrap();
    let mut root = body;
    if root.get("formatVersion").is_none() {
        root["formatVersion"] = serde_json::json!(2);
    }
    let catalog = SchemaCatalog::from_json(&root.to_string()).unwrap();
    scan(&catalog, &compiled, "schema.json")
}

pub(super) fn expect(yaml: &str, body: serde_json::Value, text: &str) {
    assert_eq!(messages(yaml, body), vec![text.to_string()]);
}

pub(super) fn expect_none(yaml: &str, body: serde_json::Value) {
    let messages = messages(yaml, body);
    assert!(messages.is_empty(), "{messages:#?}");
}

pub(super) fn expect_err(yaml: &str, snippet: &str) {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let error = match compile(&options, None) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected config error containing {snippet}"),
    };
    assert!(error.contains(snippet), "{error}");
}

pub(super) fn index(table: &str, name: &str, unique: bool, backed: bool) -> serde_json::Value {
    serde_json::json!({
        "tables": {
            table: {
                "indexes": {
                    name: { "unique": unique, "constraintBacked": backed }
                }
            }
        }
    })
}

pub(super) fn table(name: &str) -> serde_json::Value {
    serde_json::json!({ "tables": { name: {} } })
}

pub(super) fn trigger_fn(name: &str, returns: &str) -> serde_json::Value {
    let definition = format!(
        "CREATE FUNCTION {name}() RETURNS {returns} LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END; $$"
    );
    serde_json::json!({ "functions": { name: { "definition": definition } } })
}
