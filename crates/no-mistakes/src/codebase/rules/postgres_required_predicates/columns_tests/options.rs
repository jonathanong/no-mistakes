use super::super::*;
use super::{config_yaml, fixture, run, COLUMNS};

#[test]
fn repeated_scans_are_identical() {
    let root = fixture("columns");
    let sql = root.join("sql/fail-ambiguous.sql");
    let config = config_yaml(COLUMNS);
    let first = check_with_files(&root, &config, std::slice::from_ref(&sql)).unwrap();
    let second = check_with_files(&root, &config, std::slice::from_ref(&sql)).unwrap();
    assert_eq!(first, second);
}

#[test]
fn suppression_directives_hide_sql_findings() {
    let root = fixture("suppress");
    let config = config_yaml(COLUMNS);
    for name in ["sql/next-line.sql", "sql/line.sql", "sql/file.sql"] {
        let file = root.join(name);
        let mut findings = check_with_files(&root, &config, std::slice::from_ref(&file)).unwrap();
        assert!(!findings.is_empty(), "{name} {findings:?}");
        let sources = super::super::super::source_store_for_files(std::slice::from_ref(&file));
        super::super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
        assert!(findings.is_empty(), "{name} {findings:?}");
    }
}

#[test]
fn new_options_reject_invalid_values() {
    let cases = [
        (
            "partitionKeys: sideways\n",
            "option partitionKeys: expected require or off",
        ),
        (
            "partitionKeys: require\n",
            "option schemaCatalogPath: required when partitionKeys is require",
        ),
        (
            "relations:\n  - {table: orders, requireColumns: ['']}\n",
            "option requireColumns: empty column name",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\npartitionKeyExemptions:\n  - {table: '', reason: reviewed}\n",
            "option partitionKeyExemptions: empty table name",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\nallow:\n  - {object: '', reason: reviewed}\n",
            "option allow: empty object",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\npartitionKeyExemptions:\n  - {table: events, reason: ''}\n",
            "option partitionKeyExemptions: empty reason",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\npartitionKeyExemptions:\n  - {table: events, reason: a}\n  - {table: Events, reason: b}\n",
            "option partitionKeyExemptions: duplicate entry",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\nallow:\n  - {object: table:events, reason: ''}\n",
            "option allow: empty reason",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\nallow:\n  - {object: table:events, reason: a}\n  - {object: table:events, reason: b}\n",
            "option allow: duplicate entry",
        ),
    ];
    for (yaml, needle) in cases {
        let Err(error) = compile_options(&serde_yaml::from_str::<Options>(yaml).unwrap()) else {
            panic!("{yaml} compiled");
        };
        assert!(error.to_string().contains(needle), "{yaml} {error}");
    }
    assert!(compile_options(&Options {
        relations: vec![RelationOption {
            table: "orders".into(),
            require_columns: vec!["account_id".into()],
            ..Default::default()
        }],
        ..Default::default()
    })
    .is_ok());
}

#[test]
fn textual_require_message_stays_on_the_same_select() {
    let root = fixture("fail");
    let findings = check_with_files(
        &root,
        &config_yaml(
            "sqlInclude: [\"sql/**/*.sql\"]\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]\n    requireColumns: [parent_id]\n",
        ),
        &[root.join("sql/001.sql")],
    )
    .unwrap();
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("queries against topics must include `parent_id IS NOT NULL`")));
}

#[test]
fn a_loaded_catalog_skips_partition_checks_when_they_are_off() {
    let findings = run(
        "columns",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\npartitionKeys: off\nrelations:\n  - {table: orders, requireColumns: [account_id]}\n",
        "sql/fail-kind.sql",
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("partition key")),
        "{findings:?}"
    );
}
