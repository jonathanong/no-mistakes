use super::super::scan;
use super::support::{call, fragment_facts, import, insert, named, span, sql_tag};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueExpression};
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
    assert_eq!(report.findings.len(), 1, "{:?}", report.findings);
    assert_eq!(report.findings[0].file, "unread.ts");
    assert_eq!(report.findings[0].line, 1);
    assert_eq!(report.findings[0].target.as_deref(), Some("COLS"));
    assert_eq!(report.gaps.len(), 1, "{:?}", report.gaps);
    assert!(
        report.gaps[0]
            .message
            .contains("`unread.ts` could not be read"),
        "{:?}",
        report.gaps[0].message
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
