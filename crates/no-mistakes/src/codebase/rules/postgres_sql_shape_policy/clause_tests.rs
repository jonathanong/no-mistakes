use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn config(entries: &str) -> NoMistakesConfig {
    NoMistakesConfig { rules: vec![RuleDef { rule: RULE_ID.into(), scope: Some(RuleScope::Repository), options: serde_yaml::from_str(&format!("executorNames: []\nbannedShapes: [banned-function-call]\nshapeOptions:\n  bannedFunctionCall:\n    functions: {entries}")).unwrap(), ..Default::default() }], ..Default::default() }
}

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/clause-policy"),
    )
}

#[test]
fn all_scoped_clause_names_match_only_their_ast_positions() {
    let root = root();
    // Existing finding sorting deduplicates identical call diagnostics on one line.
    for (clause, count) in [
        ("where", 10),
        ("join-on", 2),
        ("having", 1),
        ("select-list", 5),
        ("order-by", 1),
        ("values", 2),
        ("set", 3),
        ("default", 2),
        ("returning", 4),
    ] {
        let findings = check_with_files(
            &root,
            &config(&format!(
                "[{{name: uuidv7, clauses: [{clause}], hint: stable bound}}]"
            )),
            &[root.join("positions.sql")],
        )
        .unwrap();
        assert_eq!(findings.len(), count, "{clause}: {findings:#?}");
        let parsed = crate::codebase::postgres::SqlFunctionClause::parse(clause).unwrap();
        assert!(
            findings.iter().all(|finding| finding
                .message
                .ends_with(&format!("is banned in {}; stable bound", parsed.label()))),
            "{findings:#?}"
        );
        let again = check_with_files(
            &root,
            &config(&format!(
                "[{{name: uuidv7, clauses: [{clause}], hint: stable bound}}]"
            )),
            &[root.join("positions.sql")],
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string(&findings).unwrap(),
            serde_json::to_string(&again).unwrap()
        );
    }
}

#[test]
fn object_without_clauses_keeps_string_diagnostics_byte_identical() {
    let root = root();
    let files = [root.join("positions.sql")];
    let strings = check_with_files(&root, &config("[uuidv7]"), &files).unwrap();
    let objects = check_with_files(&root, &config("[{name: uuidv7}]"), &files).unwrap();
    assert_eq!(
        serde_json::to_string(&strings).unwrap(),
        serde_json::to_string(&objects).unwrap()
    );
    let hinted = check_with_files(
        &root,
        &config("[{name: uuidv7, hint: choose stable}]"),
        &files,
    )
    .unwrap();
    assert!(hinted
        .iter()
        .all(|finding| finding.message.ends_with("; choose stable")));
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

#[test]
fn qualified_function_names_keep_existing_matching_and_predicate_scope() {
    let root = root();
    let findings = check_with_files(
        &root,
        &config("[{name: pg_catalog.uuidv7, clauses: [where]}]"),
        &[root.join("positions.sql")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 14);
}
