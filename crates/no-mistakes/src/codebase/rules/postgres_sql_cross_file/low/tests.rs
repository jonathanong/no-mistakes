use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use sqlparser::ast::{Ident, ObjectNamePartFunction};
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
