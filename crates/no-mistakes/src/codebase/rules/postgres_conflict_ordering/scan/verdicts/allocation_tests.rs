use super::*;
use crate::codebase::postgres::prepare_rule_sql_facts;
use crate::codebase::rules::postgres_conflict_ordering::tests::{config, fixture};
use crate::codebase::rules::postgres_conflict_ordering::{compile_options, Options};
use std::sync::Arc;

#[test]
fn disabled_ledger_constructs_no_records_and_allocates_nothing() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("pass-canonical-order"));
    let path = root.join("src/insert.ts");
    let files = [path.clone(), root.join("schema.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = prepare_rule_sql_facts(&root, &files, Arc::clone(&sources), &config(), &[RULE_ID])
        .unwrap()
        .with_graph_file_universe(files.to_vec());
    let opts = compile_options(&Options {
        schema_catalog_path: "schema.json".into(),
        import_specifier: "@example/db".into(),
        ..Default::default()
    })
    .unwrap();
    let source = crate::codebase::rules::read_source(&sources, &path).unwrap();
    let embedded = facts.embedded_sql(&path, &opts.embedded).unwrap();
    let statements = crate::codebase::rules::index_sql_variants(
        facts
            .postgres_statements(&path, Some(&opts.embedded))
            .unwrap(),
    );
    let catalog = facts.postgres_ordering_catalog("schema.json").unwrap();
    let enabled = facts.with_fresh_sql_verdicts();
    // The recorder owns its opt-in boundary, even when a future caller omits
    // the outer scan guard. Count real allocations, not estimated work sites.
    let (_, disabled_allocations) = crate::test_support::allocations::observe(|| {
        record(
            "src/insert.ts",
            &source,
            embedded,
            &opts,
            catalog,
            &statements,
            &facts,
        );
        assert!(facts.for_sql_application(0, None).is_none());
        assert!(facts.for_sql_application(1, Some("unused-name")).is_none());
    });
    assert_eq!(disabled_allocations, 0);
    assert!(facts.sql_verdicts().is_empty());
    assert!(!enabled.ts.is_empty());
    assert!(!enabled.files().is_empty());
    assert!(!enabled.graph_file_universe().is_empty());
    let compositions = crate::codebase::check_facts::CheckFactMap::view_composition_count();
    let (_, application_allocations) = crate::test_support::allocations::observe(|| {
        for index in 0..128 {
            let application = enabled
                .for_sql_application(index, Some("same-name"))
                .unwrap();
            // Identity alone must never compose or sweep another prepared map.
            assert!(std::ptr::eq(application.prepared(), &enabled));
            std::hint::black_box(application);
        }
    });
    assert_eq!(application_allocations, 0);
    assert_eq!(
        crate::codebase::check_facts::CheckFactMap::view_composition_count(),
        compositions
    );
    let application = enabled.for_sql_application(127, Some("same-name")).unwrap();
    let (_, enabled_allocations) = crate::test_support::allocations::observe(|| {
        record(
            "src/insert.ts",
            &source,
            embedded,
            &opts,
            catalog,
            &statements,
            &application,
        );
    });
    assert!(enabled_allocations > 0);
    assert_eq!(enabled.sql_verdicts().len(), embedded.calls.len());
    assert!(!embedded.calls.is_empty());
    assert!(enabled.sql_verdicts().iter().all(|verdict| {
        let application = verdict.application.as_ref().unwrap();
        application.index == 127 && application.name.as_deref() == Some("same-name")
    }));
    // The positive control still composes a genuine projection and sweeps its facts.
    let _projection = enabled.fact_view_with_supplemental(&Default::default());
    assert_eq!(
        crate::codebase::check_facts::CheckFactMap::view_composition_count(),
        compositions + 1
    );
}

#[test]
fn parallel_applications_keep_borrowed_identity_and_report_local_sinks() {
    use rayon::prelude::*;
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("pass-canonical-order"));
    let files = [root.join("src/insert.ts"), root.join("schema.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = prepare_rule_sql_facts(&root, &files, sources, &config(), &[RULE_ID]).unwrap();
    let reports = [
        facts.with_fresh_sql_verdicts(),
        facts.with_fresh_sql_verdicts(),
    ];
    // Use the real policy recorder as a positive control, under parallel, repeated names.
    let opts = compile_options(&Options {
        schema_catalog_path: "schema.json".into(),
        import_specifier: "@example/db".into(),
        ..Default::default()
    })
    .unwrap();
    let embedded = facts.embedded_sql(&files[0], &opts.embedded).unwrap();
    assert!(!embedded.calls.is_empty());
    let statements = crate::codebase::rules::index_sql_variants(
        facts
            .postgres_statements(&files[0], Some(&opts.embedded))
            .unwrap(),
    );
    let catalog = facts.postgres_ordering_catalog("schema.json").unwrap();
    (0..64).into_par_iter().for_each(|index| {
        let report = &reports[index / 32];
        let application = report
            .for_sql_application(index, Some("coincident"))
            .unwrap();
        record(
            "src/insert.ts",
            "",
            embedded,
            &opts,
            catalog,
            &statements,
            &application,
        );
    });
    assert!(facts.sql_verdicts().is_empty());
    for (report_index, report) in reports.iter().enumerate() {
        let verdicts = report.sql_verdicts();
        assert_eq!(verdicts.len(), 32 * embedded.calls.len());
        for index in report_index * 32..(report_index + 1) * 32 {
            assert_eq!(
                verdicts
                    .iter()
                    .filter(|verdict| {
                        let application = verdict.application.as_ref().unwrap();
                        application.index == index
                            && application.name.as_deref() == Some("coincident")
                    })
                    .count(),
                embedded.calls.len()
            );
        }
    }
}
