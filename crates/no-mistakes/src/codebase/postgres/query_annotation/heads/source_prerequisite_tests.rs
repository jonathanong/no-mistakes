use crate::codebase::check_facts::{
    collect_file_facts_from_program,
    sql_source_prerequisites::{SqlProjection, SqlSourceKey},
    CheckFactPlan, CheckFileFacts,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::FileIdMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn prepared() -> (FileIdMap<CheckFileFacts>, PathBuf) {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("heads-source-prerequisites");
    let profiles = ["@bench/db", "@other/db"].map(|module| {
        EmbeddedSqlOptions::configured(module, &["read".into()])
            .with_scoped_executors(&["openTransaction".into()], &[])
    });
    let mut plan = CheckFactPlan {
        query_annotation_heads: true,
        query_annotation_options: Some(profiles.to_vec()),
        embedded_sql_options: profiles.to_vec(),
        ..Default::default()
    };
    plan.graph.imports = true;
    let mut files = [
        "direct",
        "namespace",
        "async",
        "callback",
        "effect",
        "pending",
        "db",
        "healthy",
        "unused",
        "returned",
        "returned-conditional",
        "literal-helper",
        "async-effects",
        "async-factory",
    ]
    .into_iter()
    .map(|name| {
        let path = root.join(format!("{name}.mts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = crate::ast::with_program(&path, &source, |program, source| {
            collect_file_facts_from_program(&root, &path, &plan, None, source, program, None)
        })
        .unwrap();
        (path, facts)
    })
    .collect::<FileIdMap<_>>();
    let broken = root.join("broken.mts");
    let source = std::fs::read_to_string(&broken).unwrap();
    let error = crate::ast::with_program(&broken, &source, |_, _| ()).unwrap_err();
    files.insert(
        broken,
        CheckFileFacts {
            parse_error: Some(error.to_string()),
            ..Default::default()
        },
    );
    (files, root)
}

fn resolve(files: &FileIdMap<CheckFileFacts>) -> impl Fn(&str, &Path) -> Option<PathBuf> {
    let paths = files.keys().cloned().collect::<BTreeSet<_>>();
    move |specifier, from| {
        let target = from.parent()?.join(specifier).with_extension("mts");
        paths.get(&target).cloned()
    }
}

fn failures(file: &CheckFileFacts, index: usize) -> BTreeSet<PathBuf> {
    let key = SqlSourceKey::profile(SqlProjection::AnnotationHeads, &file.embedded_sql[index].0);
    file.sql_source_prerequisites
        .failures(&key)
        .cloned()
        .collect()
}

#[test]
fn direct_namespace_async_and_callback_sources_keep_exact_profile_and_primary_demand() {
    let (mut files, root) = prepared();
    for name in ["direct", "namespace", "async", "callback"] {
        let heads = files[&root.join(format!("{name}.mts"))]
            .query_annotation_heads
            .as_ref()
            .unwrap();
        let pending = match name {
            "direct" => heads.pending_imports.len(),
            "namespace" => heads.pending_namespaces.len(),
            "async" => heads.pending_async_returns.len(),
            _ => heads.pending_callbacks.len(),
        };
        assert!(pending > 0, "saved {name} must request its helper proof");
    }
    let resolver = resolve(&files);
    super::super::project(&mut files, resolver);
    let expected = [root.join("broken.mts")].into_iter().collect();
    for name in ["direct", "namespace", "async", "callback", "effect"] {
        let file = &files[&root.join(format!("{name}.mts"))];
        assert_eq!(failures(file, 0), expected, "{name}");
        assert!(failures(file, 1).is_empty(), "other profile in {name}");
    }
    for name in ["healthy", "unused"] {
        let file = &files[&root.join(format!("{name}.mts"))];
        assert!(failures(file, 0).is_empty(), "unneeded source in {name}");
        assert!(file
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .calls
            .values()
            .any(|proof| *proof == super::super::HeadProof::Literal));
    }
}

#[test]
fn effect_prerequisite_maps_to_its_executor_and_pending_calls_keep_candidate_profile() {
    let (mut files, root) = prepared();
    let effect = &files[&root.join("effect.mts")];
    let heads = effect.query_annotation_heads.as_ref().unwrap();
    assert!(!heads.pending_effects.is_empty());
    assert!(heads
        .conditional_effects
        .iter()
        .any(|condition| { !condition.required.is_empty() && !condition.spans.is_empty() }));
    let pending = &files[&root.join("pending.mts")];
    assert!(pending.embedded_sql[0].1.call_spans.is_empty());
    assert!(!pending.embedded_sql[0]
        .1
        .pending_relative
        .call_spans
        .is_empty());
    assert!(!pending
        .query_annotation_heads
        .as_ref()
        .unwrap()
        .pending_imports
        .is_empty());
    let resolver = resolve(&files);
    super::super::project(&mut files, resolver);
    assert_eq!(
        failures(&files[&root.join("pending.mts")], 0),
        [root.join("broken.mts")].into_iter().collect()
    );
    let effect = &files[&root.join("effect.mts")];
    assert_eq!(
        failures(effect, 0),
        [root.join("broken.mts")].into_iter().collect()
    );
}

#[path = "source_prerequisite_tests/extended.rs"]
mod extended;
