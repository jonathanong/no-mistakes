//! Prepared partial maps preserve advisories instead of inventing async origins.
use super::{resolve, Pending};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::query_annotation::heads::{AnnotationHeads, HeadProof};
use crate::codebase::ts_source::FileIdMap;
use std::{path::PathBuf, sync::Arc};
#[test]
fn async_projection_skips_headless_files_and_unresolved_factory_targets() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("async-return-partial/src/query.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let start = source.find("read(query)").unwrap() as u32;
    let span = (start, start + "read(query)".len() as u32);
    let mut heads = AnnotationHeads::default();
    heads.calls.insert(span, HeadProof::Unproven);
    heads.pending_async_returns.push(Pending {
        span,
        specifier: "./missing-factory.mjs".into(),
        imported: "makeQuery".into(),
    });
    let mut files = FileIdMap::from_entries([
        (
            path.clone(),
            CheckFileFacts {
                query_annotation_heads: Some(Arc::new(heads)),
                ..Default::default()
            },
        ),
        (PathBuf::from("headless.mts"), CheckFileFacts::default()),
    ]);
    let resolutions = std::cell::Cell::new(0);
    resolve::project(&mut files, &|specifier, from| {
        assert_eq!(specifier, "./missing-factory.mjs");
        assert_eq!(from, path.as_path());
        resolutions.set(resolutions.get() + 1);
        None
    });
    assert_eq!(resolutions.get(), 1);
    assert_eq!(
        files
            .get(&path)
            .unwrap()
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .calls[&span],
        HeadProof::Unproven
    );
    assert!(files
        .get(std::path::Path::new("headless.mts"))
        .unwrap()
        .query_annotation_heads
        .is_none());
}

#[test]
fn failed_second_hop_effect_is_reported_and_unresolved_effect_stays_unproven() {
    use crate::codebase::check_facts::{collect_file_facts_from_program, CheckFactPlan};
    use crate::codebase::postgres::EmbeddedSqlOptions;
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("heads-source-prerequisites");
    let profile = EmbeddedSqlOptions::configured("@bench/db", &["read".into()]);
    let mut plan = CheckFactPlan {
        query_annotation_heads: true,
        query_annotation_options: Some(vec![profile.clone()]),
        embedded_sql_options: vec![profile],
        ..Default::default()
    };
    plan.graph.imports = true;
    let prepared = ["async-effects.mts", "async-factory.mts"].map(|name| {
        let path = root.join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let file = crate::ast::with_program(&path, &source, |program, source| {
            collect_file_facts_from_program(&root, &path, &plan, None, source, program, None)
        })
        .unwrap();
        (path, file)
    });
    let mut files = FileIdMap::from_entries(prepared);
    let caller = root.join("async-effects.mts");
    let factory = root.join("async-factory.mts");
    assert_eq!(
        files[&caller]
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .pending_async_returns
            .len(),
        1
    );
    let summary = &files[&factory]
        .query_annotation_heads
        .as_ref()
        .unwrap()
        .exported_async_returns["factory"];
    assert_eq!(summary.required.len(), 1);
    let broken = root.join("broken.mts");
    files.insert(
        broken.clone(),
        CheckFileFacts {
            parse_error: Some("saved failed helper".into()),
            ..Default::default()
        },
    );
    let failures = resolve::project(&mut files, &|specifier, _| match specifier {
        "./async-factory.mjs" => Some(factory.clone()),
        "./broken.mjs" => Some(broken.clone()),
        _ => None,
    });
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].2, [broken].into_iter().collect());
    let failures = resolve::project(&mut files, &|specifier, _| {
        (specifier == "./async-factory.mjs").then(|| factory.clone())
    });
    assert!(failures.is_empty());
}
