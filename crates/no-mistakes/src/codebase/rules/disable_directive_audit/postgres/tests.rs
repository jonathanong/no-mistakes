use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::sql_identifier_syntax::{
    helper_recovery::ApplicationKey, Input, SqlCall,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::sync::Arc;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/unused-postgres-disable-directives/fixture")
}

fn sql() -> &'static str {
    include_str!("../../../../../../../test-cases/rules/unused-postgres-disable-directives/fixture/probe.sql")
}

#[test]
fn postgres_directive_locations_require_selected_readable_matching_physical_comments() {
    let root = root();
    let path = root.join("probe.sql");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let selected = BTreeSet::from([path.clone()]);
    let scope = AuditScope {
        root: &root,
        rule: "postgres-required-predicates",
        selected: &selected,
        sources: &sources,
    };
    let mut comments = Vec::new();
    let text = |base| SqlText {
        sql: sql(),
        positions: &[],
        base,
    };
    collect(&mut comments, &scope, &path, text(1));
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].directive.line, 1);
    comments.clear();
    collect(&mut comments, &scope, &path, text(2));
    assert!(comments.is_empty());
    collect(&mut comments, &scope, &path, SqlText {
        sql: include_str!("../../../../../../../test-cases/rules/unused-postgres-disable-directives/fixture/clean.sql"),
        positions: &[], base: 1,
    });
    assert!(comments.is_empty());
    let excluded = AuditScope {
        selected: &BTreeSet::new(),
        ..scope
    };
    collect(&mut comments, &excluded, &path, text(1));
    assert!(comments.is_empty());
    let missing = root.join("missing.sql");
    let missing_scope = AuditScope {
        selected: &BTreeSet::from([missing.clone()]),
        ..scope
    };
    collect(&mut comments, &missing_scope, &missing, text(1));
    assert!(comments.is_empty());
}

#[test]
fn postgres_low_directive_audit_skips_missing_fact_profiles_and_source_inputs() {
    let root = root();
    let path = root.join("missing.ts");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let selected = BTreeSet::from([path.clone()]);
    let scope = AuditScope {
        root: &root,
        rule: "postgres-required-predicates",
        selected: &selected,
        sources: &sources,
    };
    let profile = EmbeddedSqlOptions::default();
    let key = ApplicationKey::new(&profile, scope.rule, 0);
    let mut facts = CheckFactMap::default();
    let mut comments = Vec::new();
    low::collect(&mut comments, &scope, &facts, &path, &key);
    assert!(comments.is_empty());
    facts
        .ts
        .insert(path.clone(), Arc::new(CheckFileFacts::default()));
    low::collect(&mut comments, &scope, &facts, &path, &key);
    assert!(comments.is_empty());
    let mut file = CheckFileFacts::default();
    file.sql_identifier_syntax.profiles.push((
        profile,
        vec![SqlCall {
            span: (0, 1),
            line: 1,
            input: Input::Sql {
                text: sql().into(),
                source_span: (0, 1),
                holes: Vec::new(),
                parameterized: None,
                positions: Vec::new(),
            },
        }],
    ));
    facts.ts.insert(path.clone(), Arc::new(file));
    low::collect(&mut comments, &scope, &facts, &path, &key);
    assert!(comments.is_empty());
}
