use super::super::compile::compile;
use super::super::scan::scan;
use super::super::Options;
use crate::codebase::postgres::SchemaCatalog;
use crate::codebase::rules::RuleFinding;

pub(super) const PROPOSED: &str = r#"
schemaCatalogPath: db/schema.json
typeRules:
  - types: ['timestamp with time zone', 'timestamp without time zone']
    namePattern: '_at$'
    hint: 'end timestamp columns in _at'
  - types: ['date']
    namePattern: '(^day|_on)$'
    hint: 'name date columns day or end them in _on'
  - types: ['boolean']
    namePattern: '(^|_)(is|has|can|should)_'
    hint: 'name boolean columns as a predicate: is_, has_, can_ or should_'
nameTypeRules:
  - namePattern: '_at$'
    types: ['timestamp with time zone']
    hint: 'a column ending in _at holds a timestamp with time zone'
  - namePattern: '^cursor_.*_id$'
    types: [uuid]
    hint: "a keyset cursor holds the swept table's uuid primary key"
skipGeneratedColumns: false
ignoreTablePatterns: ['^link__']
forbiddenColumnNames:
  - pattern: '(^|_)table(_name)?$'
    hint: 'a column must not name a table; use one table per target, or an enum'
foreignKeys:
  targetSuffixes:
    - tables: [users, deleted_user_identities]
      suffixes: ['_user_id', '_by_id']
  reservedSuffixes:
    - suffix: '_user_id'
      types: [uuid]
      tables: [users, deleted_user_identities]
    - suffix: '_by_id'
      types: [uuid]
      tables: [users, deleted_user_identities]
    - suffix: '_url'
      types: [text, 'character varying']
      tables: [links]
      hint: 'store the URL in links and reference it with a <name>_link_id column'
  targetMatch: last-word
  targetNames:
    - tablePattern: '^archived_(.+)_snapshots$'
      name: '$1'
  checkSelfReferences: false
  singular: { people: person, aliases: alias, statuses: status }
  followCompositeForeignKeys: true
  requireForeignKey:
    types: [uuid]
    namePattern: '_id$'
    exempt:
      - namePattern: '^cursor_'
        reason: 'Keyset cursors may point at rows that are already deleted'
      - namePattern: '(^|_)(device|session)_id$'
        reason: 'Device and session ids are issued in signed tokens, not stored in a table'
allow:
  - object: 'column:sessions.expires'
    reason: 'Name mirrors an external protocol field'
  - object: 'column:links.url'
    reason: 'The URL registry itself; every other URL references this row'
  - object: 'column:export_jobs.runner_job_id'
    reason: 'Id assigned by the external job runner; there is no local row'
"#;

pub(super) const TYPES: &str = r#"
schemaCatalogPath: schema.json
typeRules:
  - types: ['timestamp with time zone', 'timestamp without time zone']
    namePattern: '_at$'
    hint: 'end timestamp columns in _at'
  - types: ['date']
    namePattern: '(^day|_on)$'
    hint: 'name date columns day or end them in _on'
  - types: ['boolean']
    namePattern: '(^|_)(is|has|can|should)_'
    hint: 'name boolean columns as a predicate: is_, has_, can_ or should_'
nameTypeRules:
  - namePattern: '_at$'
    types: ['timestamp with time zone']
    hint: 'a column ending in _at holds a timestamp with time zone'
  - namePattern: '^cursor_.*_id$'
    types: [uuid]
    hint: "a keyset cursor holds the swept table's uuid primary key"
"#;

pub(super) const SUFFIX: &str = r#"
schemaCatalogPath: schema.json
foreignKeys:
  targetSuffixes:
    - tables: [users, deleted_user_identities]
      suffixes: ['_user_id', '_by_id']
  targetMatch: last-word
  singular: { people: person, aliases: alias, statuses: status }
"#;

pub(super) const LAST_WORD: &str = r#"
schemaCatalogPath: schema.json
foreignKeys:
  targetMatch: last-word
  singular: { people: person, aliases: alias, statuses: status }
  targetNames:
    - tablePattern: '^archived_(.+)_snapshots$'
      name: '$1'
"#;

pub(super) const RESERVED: &str = r#"
schemaCatalogPath: schema.json
foreignKeys:
  reservedSuffixes:
    - suffix: '_user_id'
      types: [uuid]
      tables: [users, deleted_user_identities]
    - suffix: '_by_id'
      types: [uuid]
      tables: [users, deleted_user_identities]
    - suffix: '_url'
      types: [text, 'character varying']
      tables: [links]
      hint: 'store the URL in links and reference it with a <name>_link_id column'
  followCompositeForeignKeys: false
"#;

pub(super) const REQUIRE: &str = r#"
schemaCatalogPath: schema.json
foreignKeys:
  requireForeignKey:
    types: [uuid]
    namePattern: '_id$'
"#;

pub(super) fn messages(yaml: &str, body: serde_json::Value) -> Vec<String> {
    findings(yaml, body)
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

pub(super) fn findings(yaml: &str, body: serde_json::Value) -> Vec<RuleFinding> {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let compiled = compile(&options).unwrap();
    let mut root = body;
    if root.get("formatVersion").is_none() {
        root["formatVersion"] = serde_json::json!(2);
    }
    let catalog = SchemaCatalog::from_json(&root.to_string()).unwrap();
    scan(&catalog, &compiled, &options.schema_catalog_path)
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
    let error = match compile(&options) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected config error containing {snippet}"),
    };
    assert!(error.contains(snippet), "{error}");
}

pub(super) fn column(table: &str, name: &str, data_type: &str) -> serde_json::Value {
    serde_json::json!({
        "tables": { table: { "columns": { name: { "dataType": data_type } } } }
    })
}

pub(super) fn sole_fk(
    table: &str,
    name: &str,
    data_type: &str,
    referenced: &str,
    referenced_column: &str,
) -> serde_json::Value {
    serde_json::json!({
        "tables": {
            table: {
                "columns": { name: { "dataType": data_type } },
                "foreignKeys": {
                    "fk": {
                        "columns": [name],
                        "referencedTable": referenced,
                        "referencedColumns": [referenced_column]
                    }
                }
            }
        }
    })
}

pub(super) fn at(table: &str, column: &str, text: &str) -> String {
    format!("schema.json: column:{table}.{column}: {text}")
}
