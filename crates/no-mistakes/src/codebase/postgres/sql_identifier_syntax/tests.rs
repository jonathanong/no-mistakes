use super::*;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use std::path::PathBuf;

fn fixture(name: &str, visit: impl FnOnce(&Program<'_>, &str, TsFileFacts)) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/effort-low/src")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        visit(program, &source, ts);
    })
    .unwrap();
}
#[test]
fn low_syntax_executor_alias_is_unknown_only_at_configured_call_sites() {
    fixture("executor-alias.ts", |program, source, ts| {
        let selected = EmbeddedSqlOptions::configured("@example/db", &[]);
        let unrelated = EmbeddedSqlOptions::configured("@unrelated/db", &[]);
        let result = collect(program, &ts, source, &[selected, unrelated]);
        let calls = &result.profiles[0].1;
        assert_eq!(
            calls.iter().map(|call| call.line).collect::<Vec<_>>(),
            vec![4, 6]
        );
        assert!(calls
            .iter()
            .all(|call| matches!(call.input, Input::Unknown)));
        assert!(result.profiles[1].1.is_empty());
    });
    fixture("executor-alias-escape.ts", |program, source, ts| {
        let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
        let result = collect(program, &ts, source, &[profile]);
        assert_eq!(result.profiles[0].1.len(), 1);
        assert_eq!(result.profiles[0].1[0].line, 3);
        assert!(matches!(result.profiles[0].1[0].input, Input::Unknown));
    });
}
#[test]
fn low_syntax_recovery_selects_only_configured_imports() {
    fixture("selection.ts", |program, source, _| {
        let target = EmbeddedSqlOptions::configured("@example/db", &[]);
        let unrelated = EmbeddedSqlOptions::configured("@unrelated/db", &[]);
        let result = collect_invalid(
            program,
            source,
            &[target, unrelated, EmbeddedSqlOptions::default()],
        );
        assert!(result.profiles[0].1.len() >= 2);
        assert!(result.profiles[0]
            .1
            .iter()
            .all(|call| matches!(call.input, Input::Unknown)));
        assert!(result.profiles[1].1.is_empty());
        assert!(result.profiles[2].1.is_empty());
    });
    fixture("recovery-imports.ts", |program, source, _| {
        let profile = EmbeddedSqlOptions::configured("@example/db", &[])
            .with_scoped_executors(&["createExecutor".into()], &["Executor".into()]);
        assert_eq!(
            collect_invalid(program, source, &[profile]).profiles[0]
                .1
                .len(),
            5
        );
    });
}
#[test]
fn low_syntax_missing_semantics_never_create_positive_proofs() {
    fixture("query.ts", |program, source, mut ts| {
        let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
        let empty = collect(
            program,
            &TsFileFacts::default(),
            source,
            std::slice::from_ref(&profile),
        );
        assert!(empty.profiles.is_empty());
        std::sync::Arc::make_mut(ts.module_bindings.as_mut().unwrap())
            .diagnostics
            .push(crate::codebase::ts_source::module_facts::ModuleDiagnostic {
                kind: "unsupported".into(),
                message: "binding semantics unavailable".into(),
                span: None,
            });
        let result = collect(
            program,
            &ts,
            source,
            &[profile, EmbeddedSqlOptions::default()],
        );
        assert!(result.profiles[0]
            .1
            .iter()
            .all(|call| matches!(call.input, Input::Unknown)));
        assert!(result.profiles[1].1.is_empty());
    });
}
#[test]
fn low_syntax_broad_member_selection_is_unknown_and_profile_local() {
    fixture("selection.ts", |program, source, ts| {
        let profile = EmbeddedSqlOptions::configured("", &["query".into()]);
        let unrelated = EmbeddedSqlOptions::configured("@unrelated/db", &[]);
        let result = collect(program, &ts, source, &[profile, unrelated]);
        assert!(!result.profiles[0].1.is_empty());
        assert!(result.profiles[0]
            .1
            .iter()
            .any(|call| matches!(call.input, Input::Unknown)));
        assert!(result.profiles[1].1.is_empty());
    });
}

#[test]
fn low_syntax_scope_union_keeps_distinct_applications_once() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/effort-low");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule = config.rule_applications("postgres-no-unbounded-sql-identifier")[0];
    let first =
        plan::SqlIdentifierScan::new(&root, &config, rule, 0, &["src/query.ts".into()], &[])
            .unwrap();
    let second =
        plan::SqlIdentifierScan::new(&root, &config, rule, 1, &["src/selection.ts".into()], &[])
            .unwrap();
    assert!(first.includes(&root, &root.join("src/query.ts")));
    assert!(!first.includes(&root, &root.join("src/selection.ts")));
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    let mut existing = plan::SqlIdentifierScans::new();
    existing.insert(profile.clone(), vec![first.clone()]);
    let mut additional = plan::SqlIdentifierScans::new();
    additional.insert(profile.clone(), vec![first, second]);
    plan::merge(&mut existing, additional);
    assert_eq!(existing[&profile].len(), 2);
    assert!(existing[&profile][1].includes(&root, &root.join("src/selection.ts")));
}
