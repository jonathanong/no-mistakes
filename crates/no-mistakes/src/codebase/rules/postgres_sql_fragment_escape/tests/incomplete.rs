use super::super::scan;
use super::support::{call, fragment_facts, import, insert, named, span, sql_tag};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBindingKind, ValueExpression,
};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn incomplete_consumer_still_reports_a_known_append() {
    let partial = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/edges/partial"),
    );
    let columns = partial.join("columns.ts");
    let mutator = partial.join("mut.ts");
    let source = std::fs::read_to_string(&mutator).expect("saved mut.ts");
    let mut producer = fragment_facts(&[("COLS", 10)]);
    producer.operational_error = Some("unreadable".to_string());
    let mut consumer = crate::codebase::ts_source::facts::TsFileFacts {
        source: Some(Arc::<str>::from(source.as_str())),
        ..Default::default()
    };
    consumer.value_flow.incomplete_reason = Some("partial".to_string());
    consumer.imported_bindings.push(import(
        "./columns",
        "COLS",
        "COLS",
        ImportedBindingKind::Named,
    ));
    let append = span(&source, "COLS.append(extra)");
    consumer.value_flow.calls.push(call(
        append.0,
        append.1,
        ValueExpression::Member(Arc::new(named("COLS")), "append".to_string()),
        vec![ValueExpression::Unknown],
    ));
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    insert(&mut facts, columns, producer);
    insert(&mut facts, mutator, consumer);
    let graph = from_raw_maps(partial.clone(), HashMap::new(), HashMap::new());
    let report = scan::scan(&partial, &[sql_tag()], Some(&graph), &facts, None, None);
    let findings = &report.findings;
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "mut.ts");
    assert_eq!(findings[0].line, 4);
    assert_eq!(findings[0].target.as_deref(), Some("COLS"));
    assert_eq!(
        findings[0].message,
        "exported SQL fragment `COLS` from `columns.ts` is written, rebound, or escaped"
    );
    let messages: Vec<&str> = report.gaps.iter().map(|gap| gap.message.as_str()).collect();
    assert!(
        messages.iter().any(|message| message.contains("columns.ts")
            && message.contains("operational error (unreadable)")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("mut.ts") && message.contains("incomplete (partial)")),
        "{messages:?}"
    );
}

#[test]
fn fatal_parse_error_gaps_without_dropping_a_known_append() {
    let partial = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/edges/partial"),
    );
    let columns = partial.join("columns.ts");
    let mutator = partial.join("mut.ts");
    let source = std::fs::read_to_string(&mutator).expect("saved mut.ts");
    let producer = fragment_facts(&[("COLS", 10)]);
    let mut consumer = crate::codebase::ts_source::facts::TsFileFacts {
        source: Some(Arc::<str>::from(source.as_str())),
        fatal_parse_error: true,
        ..Default::default()
    };
    consumer.imported_bindings.push(import(
        "./columns",
        "COLS",
        "COLS",
        ImportedBindingKind::Named,
    ));
    let append = span(&source, "COLS.append(extra)");
    consumer.value_flow.calls.push(call(
        append.0,
        append.1,
        ValueExpression::Member(Arc::new(named("COLS")), "append".to_string()),
        vec![ValueExpression::Unknown],
    ));
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    insert(&mut facts, columns, producer);
    insert(&mut facts, mutator, consumer);
    let graph = from_raw_maps(partial.clone(), HashMap::new(), HashMap::new());
    let report = scan::scan(&partial, &[sql_tag()], Some(&graph), &facts, None, None);
    assert_eq!(report.findings.len(), 1, "{:?}", report.findings);
    assert_eq!(report.findings[0].file, "mut.ts");
    assert_eq!(report.findings[0].target.as_deref(), Some("COLS"));
    assert_eq!(report.gaps.len(), 1, "{:?}", report.gaps);
    assert!(
        report.gaps[0]
            .message
            .contains("`mut.ts` has a fatal parse error"),
        "{:?}",
        report.gaps[0].message
    );
}

#[test]
fn unreadable_consumer_still_reports_a_recorded_append() {
    let partial = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/edges/partial"),
    );
    let columns = partial.join("columns.ts");
    let unread = partial.join("unread.ts");
    let producer = fragment_facts(&[("COLS", 10)]);
    let mut consumer = crate::codebase::ts_source::facts::TsFileFacts::default();
    consumer.imported_bindings.push(import(
        "./columns",
        "COLS",
        "COLS",
        ImportedBindingKind::Named,
    ));
    consumer.value_flow.calls.push(call(
        0,
        16,
        ValueExpression::Member(Arc::new(named("COLS")), "append".to_string()),
        vec![ValueExpression::Unknown],
    ));
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    insert(&mut facts, columns, producer);
    insert(&mut facts, unread, consumer);
    let graph = from_raw_maps(partial.clone(), HashMap::new(), HashMap::new());
    let report = scan::scan(&partial, &[sql_tag()], Some(&graph), &facts, None, None);
    assert!(
        report.findings.is_empty(),
        "an unreadable file is not reported on line 1: {:?}",
        report.findings
    );
    assert_eq!(report.gaps.len(), 1, "{:?}", report.gaps);
    assert!(
        report.gaps[0]
            .message
            .contains("`unread.ts` could not be read"),
        "{:?}",
        report.gaps[0].message
    );
}

pub(super) fn fact_miss_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/fact-miss"),
    )
}

pub(super) fn scan_fact_miss(
    file_name: &str,
    consumer: crate::codebase::ts_source::facts::TsFileFacts,
) -> scan::ScanReport {
    scan_fact_miss_at(fact_miss_root(), file_name, consumer)
}

fn scan_fact_miss_at(
    root: PathBuf,
    file_name: &str,
    consumer: crate::codebase::ts_source::facts::TsFileFacts,
) -> scan::ScanReport {
    let file = root.join(file_name);
    let columns = root.join("columns.ts");
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    insert(
        &mut facts,
        columns,
        fragment_facts(&[("widgetColumns", 10)]),
    );
    insert(&mut facts, file, consumer);
    let graph = from_raw_maps(root.clone(), HashMap::new(), HashMap::new());
    scan::scan(&root, &[sql_tag()], Some(&graph), &facts, None, None)
}

#[test]
fn missing_export_offset_is_a_gap() {
    let root = fact_miss_root().parent().unwrap().join("literal-positions");
    let source =
        std::fs::read_to_string(root.join("sourced-reexport.ts")).expect("saved reexport.ts");
    let mut consumer = crate::codebase::ts_source::facts::TsFileFacts {
        source: Some(Arc::<str>::from(source.as_str())),
        ..Default::default()
    };
    consumer.exported_bindings.push(ExportedBinding {
        specifier: Some("./columns".to_string()),
        local: "widgetColumns".to_string(),
        exported: "itemColumns".to_string(),
    });
    let report = scan_fact_miss_at(root, "sourced-reexport.ts", consumer);
    assert!(
        report.findings.is_empty(),
        "a missing sourced export offset is not placed by scanning source: {:?}",
        report.findings
    );
    assert!(
        report.gaps.iter().any(|gap| {
            gap.message.contains("`sourced-reexport.ts`")
                && gap.message.contains("export offset for `widgetColumns`")
        }),
        "{:?}",
        report.gaps
    );
}

#[test]
fn recorded_export_offset_reports_the_reexport() {
    let root = fact_miss_root();
    let source = std::fs::read_to_string(root.join("reexport.ts")).expect("saved reexport.ts");
    let offset = span(&source, "export {").0;
    let mut consumer = crate::codebase::ts_source::facts::TsFileFacts {
        source: Some(Arc::<str>::from(source.as_str())),
        ..Default::default()
    };
    consumer.imported_bindings.push(import(
        "./columns",
        "widgetColumns",
        "widgetColumns",
        ImportedBindingKind::Named,
    ));
    consumer.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "widgetColumns".to_string(),
        exported: "itemColumns".to_string(),
    });
    consumer
        .namespaces
        .export_name_offsets
        .push(("widgetColumns".to_string(), offset));
    let report = scan_fact_miss("reexport.ts", consumer);
    assert_eq!(report.findings.len(), 1, "{:?}", report.findings);
    assert_eq!(report.findings[0].file, "reexport.ts");
    assert_eq!(report.findings[0].line, 3);
    assert_eq!(report.findings[0].target.as_deref(), Some("widgetColumns"));
    assert!(
        report
            .gaps
            .iter()
            .all(|gap| !gap.message.contains("export offset")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn missing_static_import_span_is_not_recovered_from_text() {
    for file_name in ["static-import.ts", "static-require.ts"] {
        let root = fact_miss_root();
        let source = std::fs::read_to_string(root.join(file_name)).expect(file_name);
        let mut consumer = crate::codebase::ts_source::facts::TsFileFacts {
            source: Some(Arc::<str>::from(source.as_str())),
            ..Default::default()
        };
        consumer.imported_bindings.push(import(
            "./columns",
            "widgetColumns",
            "widgetColumns",
            ImportedBindingKind::Named,
        ));
        consumer.value_flow.bindings.push(super::support::bind(
            "parts",
            0,
            ValueExpression::Unknown,
        ));
        let append = span(&source, "parts.append(extra)");
        consumer.value_flow.calls.push(call(
            append.0,
            append.1,
            ValueExpression::Member(Arc::new(named("parts")), "append".to_string()),
            vec![ValueExpression::Unknown],
        ));
        let report = scan_fact_miss(file_name, consumer);
        assert!(
            report.findings.is_empty(),
            "{file_name}: a missing import specifier is not recovered from source: {:?}",
            report.findings
        );
        assert!(
            report.gaps.iter().any(|gap| {
                gap.message.contains(&format!("`{file_name}`"))
                    && gap.message.contains("no recorded specifier")
            }),
            "{file_name}: {:?}",
            report.gaps
        );
    }
}

#[test]
fn recorded_static_import_span_reports_the_append() {
    let root = fact_miss_root();
    let source =
        std::fs::read_to_string(root.join("static-import.ts")).expect("saved static-import.ts");
    let import_at = span(&source, "import(").0;
    let mut consumer = crate::codebase::ts_source::facts::TsFileFacts {
        source: Some(Arc::<str>::from(source.as_str())),
        ..Default::default()
    };
    consumer
        .namespaces
        .literal_import_spans
        .push((import_at, "./columns".to_string()));
    consumer
        .value_flow
        .bindings
        .push(super::support::bind("parts", 0, ValueExpression::Unknown));
    let append = span(&source, "parts.append(extra)");
    consumer.value_flow.calls.push(call(
        append.0,
        append.1,
        ValueExpression::Member(Arc::new(named("parts")), "append".to_string()),
        vec![ValueExpression::Unknown],
    ));
    let report = scan_fact_miss("static-import.ts", consumer);
    assert_eq!(report.findings.len(), 1, "{:?}", report.findings);
    assert_eq!(report.findings[0].file, "static-import.ts");
    assert_eq!(report.findings[0].target.as_deref(), Some("widgetColumns"));
    assert!(
        report
            .gaps
            .iter()
            .all(|gap| !gap.message.contains("no recorded specifier")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn only_reached_missing_facts_and_operational_failures_produce_gaps() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/dependency-gaps"),
    );
    let reached = root.join("query-broken.ts");
    let unrelated = root.join("unrelated-broken.ts");
    let operational = root.join("reached-unreadable.ts");
    let unrelated_operational = root.join("unrelated-unreadable.ts");
    assert!(!operational.exists() && !unrelated_operational.exists());
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        graph_files: vec![reached.clone(), unrelated, root.join("tsconfig.json")],
        ..CheckFactMap::default()
    };
    let mut producer = fragment_facts(&[("COLS", 10)]);
    producer.imported_bindings.push(import(
        "./query-broken",
        "first",
        "first",
        ImportedBindingKind::Named,
    ));
    producer.imported_bindings.push(import(
        "./reached-unreadable",
        "second",
        "second",
        ImportedBindingKind::Named,
    ));
    insert(&mut facts, root.join("columns.ts"), producer);
    for file in [&operational, &unrelated_operational] {
        insert(
            &mut facts,
            file.to_path_buf(),
            crate::codebase::ts_source::facts::TsFileFacts {
                operational_error: Some("prepared reader failed".to_owned()),
                ..Default::default()
            },
        );
    }
    let report = scan::scan(&root, &[sql_tag()], None, &facts, None, None);
    assert!(report.findings.is_empty());
    assert_eq!(report.gaps.len(), 3, "{:?}", report.gaps);
    assert!(report
        .gaps
        .iter()
        .any(|gap| gap.file.as_deref() == Some(reached.as_path())
            && gap.message.contains("missing prepared TypeScript facts")));
    assert!(report
        .gaps
        .iter()
        .any(|gap| gap.file.as_deref() == Some(operational.as_path())
            && gap
                .message
                .contains("operational error (prepared reader failed)")));
    assert!(report
        .gaps
        .iter()
        .any(|gap| gap.file.as_deref() == Some(operational.as_path())
            && gap.message.contains("could not be read")));
}
