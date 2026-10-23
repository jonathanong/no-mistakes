use crate::codebase::check_facts::{collect_file_facts_from_program, CheckFactPlan};
use crate::codebase::postgres::{EmbeddedSqlKind, EmbeddedSqlOptions};
use std::path::PathBuf;

fn prepared(
    name: &str,
    profile: &EmbeddedSqlOptions,
    selected: &EmbeddedSqlOptions,
) -> crate::codebase::check_facts::CheckFileFacts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let plan = CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(vec![selected.clone()]),
        embedded_sql_options: vec![profile.clone()],
        ..Default::default()
    };
    crate::ast::with_program(&path, &source, |program, source| {
        collect_file_facts_from_program(
            path.parent().unwrap(),
            &path,
            &plan,
            None,
            source,
            program,
            None,
        )
    })
    .unwrap()
}

#[test]
fn max_proofs_require_trusted_append_origins_and_safe_local_uses() {
    let profile = EmbeddedSqlOptions::configured("@bench/db", &["read".into()]);
    let file = prepared("annotation-trusted-append.mts", &profile, &profile);
    let embedded = &file.embedded_sql[0].1;
    assert_eq!(embedded.calls.len(), 7);
    let heads = file.query_annotation_heads.as_ref().unwrap();
    for index in [0, 6] {
        let call = &embedded.calls[index];
        assert_eq!(call.variants.len(), 1);
        assert_eq!(call.variants[0].sql_text, call.sql_text.as_deref().unwrap());
        assert!(!call.variants[0].append_sites.is_empty());
    }
    assert_eq!(embedded.calls[1].kind, EmbeddedSqlKind::Composed);
    assert_eq!(embedded.calls[2].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        embedded
            .call_spans
            .iter()
            .map(|span| heads.accepts_trusted_append(*span))
            .collect::<Vec<_>>(),
        // Shared singleton facts already carry the trusted positive proof.
        [false, false, false, false, false, false, false]
    );
}

#[test]
fn unselected_max_profiles_do_not_collect_head_facts_or_lexical_indexes() {
    let profile = EmbeddedSqlOptions::configured("@bench/db", &["read".into()]);
    let selected = EmbeddedSqlOptions::configured("@other/db", &["read".into()]);
    let file = prepared("annotation-trusted-append.mts", &profile, &selected);
    assert!(file.query_annotation_heads.is_none());
    assert!(file.ts.module_bindings.is_none());
}

#[test]
fn ordinary_inline_calls_need_no_max_append_head_pass() {
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    let file = prepared("inline-insert.ts", &profile, &profile);
    assert!(!file.embedded_sql[0].1.calls.is_empty());
    assert!(file.query_annotation_heads.is_none());
    assert!(file.ts.module_bindings.is_none());
}

#[test]
fn trusted_tagged_builder_origins_are_required_for_composed_head_proofs() {
    let profile = EmbeddedSqlOptions::configured("@bench/db", &["read".into()]);
    let file = prepared("annotation-trusted-append.mts", &profile, &profile);
    let embedded = &file.embedded_sql[0].1;
    let candidates = embedded.call_spans.iter().copied().collect();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/annotation-trusted-append.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let heads = crate::ast::with_program(&path, &source, |program, _| {
        super::super::collect_with_trusted_appends(
            program,
            &file.ts,
            embedded.call_spans.iter().copied(),
            &candidates,
            false,
        )
    })
    .unwrap();
    let is_trusted = |index| heads.accepts_trusted_append(embedded.call_spans[index]);
    assert!(is_trusted(0), "default SQL tag with a safe local append");
    assert!(
        is_trusted(6),
        "renamed default SQL tag with a safe local append"
    );
    for index in [1, 2, 3, 4, 5] {
        assert!(
            !is_trusted(index),
            "untrusted or escaped builder at {index}"
        );
    }
}
