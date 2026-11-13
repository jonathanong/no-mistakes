use super::{recovered::Relations, *};
use crate::codebase::check_facts::CheckFileFacts;
use sqlparser::ast::{Ident, ObjectName, ObjectNamePart, ObjectNamePartFunction, Visitor};
use std::{path::PathBuf, sync::Arc};

#[test]
fn low_missing_prepared_facts_fail_but_unselected_parser_recovery_has_no_sql_proof() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/effort-low");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule_id = "postgres-no-unbounded-sql-identifier";
    let application = config.rule_applications(rule_id)[0];
    let path = root.join("src/query.ts");
    let mut facts = CheckFactMap {
        files: vec![path.clone()],
        ..Default::default()
    };
    assert!(check(&root, &config, application, 0, &facts, rule_id)
        .unwrap_err()
        .to_string()
        .contains("prepared facts are missing"));
    facts
        .ts
        .insert(path.clone(), Arc::new(CheckFileFacts::default()));
    assert!(check(&root, &config, application, 0, &facts, rule_id)
        .unwrap_err()
        .to_string()
        .contains("syntax projection is missing"));
    facts.ts.insert(
        path,
        Arc::new(CheckFileFacts {
            parse_error: Some("unrecoverable syntax".into()),
            ..Default::default()
        }),
    );
    assert!(check(&root, &config, application, 0, &facts, rule_id)
        .unwrap()
        .is_empty());
}

#[test]
fn identifier_proof_ignores_dialect_function_object_names() {
    let mut relations = Relations::default();
    let name = ObjectName(vec![ObjectNamePart::Function(ObjectNamePartFunction {
        name: Ident::new("identifier"),
        args: Vec::new(),
    })]);
    assert!(relations.pre_visit_relation(&name).is_continue());
    assert!(relations.names.is_empty());
}

#[test]
fn low_check_rejects_option_glob_and_policy_failures_before_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/effort-low");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule_id = "postgres-no-unbounded-sql-identifier";
    let base = config.rule_applications(rule_id)[0].clone();
    let facts = CheckFactMap::default();
    for (options, expected) in [
        (
            "include: 1\nimportSpecifier: \"@example/db\"\n",
            "invalid options",
        ),
        (
            "exclude: ['[']\nimportSpecifier: \"@example/db\"\n",
            "exclude",
        ),
        (
            "importSpecifier: \"@example/db\"\ninclude: [\"src/**/*.ts\"]\nexclude: [\"src/excluded.ts\"]\nunanalyzable: 1\n",
            "unanalyzable",
        ),
    ] {
        let mut application = base.clone();
        application.resolved_analysis = None;
        application.options = serde_yaml::from_str(options).unwrap();
        let error = check(&root, &config, &application, 0, &facts, rule_id)
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
    let mut application = base;
    application.include = vec!["[".into()];
    let error = check(&root, &config, &application, 0, &facts, rule_id)
        .unwrap_err()
        .to_string();
    assert!(error.contains("include"), "{error}");
}
