use super::*;
use crate::codebase::postgres::query_annotation::heads::{effects::Conditional, AnnotationHeads};
use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::sync::Arc;

#[test]
fn conditional_effect_failure_reaches_its_executor_only() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("heads-source-prerequisites");
    let owner = root.join("effect.mts");
    let source = std::fs::read_to_string(&owner).unwrap();
    let span = |text: &str| {
        let start = source.find(text).unwrap() as u32;
        (start, start + text.len() as u32)
    };
    let required = span("add(statement, tail)");
    let executor = span("read(statement)");
    let profile = EmbeddedSqlOptions::configured("@bench/db", &["read".into()]);
    let embedded = extract_embedded_sql_from_source(&owner, &source, &profile);
    assert!(embedded.call_spans.contains(&executor));
    let heads = AnnotationHeads {
        conditional_effects: vec![Conditional {
            required: vec![required],
            spans: vec![executor],
        }],
        ..Default::default()
    };
    let mut files = FileIdMap::from_entries([(
        owner.clone(),
        CheckFileFacts {
            query_annotation_heads: Some(Arc::new(heads)),
            embedded_sql: vec![(profile.clone(), embedded)],
            ..Default::default()
        },
    )]);
    let failed = root.join("broken.mts");
    record(&mut files, &owner, required, Default::default());
    record(
        &mut files,
        &root.join("absent.mts"),
        required,
        [failed.clone()].into_iter().collect(),
    );
    assert!(files[&owner].sql_source_prerequisites.is_empty());
    record(
        &mut files,
        &owner,
        required,
        [failed.clone()].into_iter().collect(),
    );
    let key = SqlSourceKey::profile(SqlProjection::AnnotationHeads, &profile);
    assert_eq!(
        files[&owner]
            .sql_source_prerequisites
            .failures(&key)
            .cloned()
            .collect::<Vec<_>>(),
        vec![failed]
    );
}
