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
    let findings = scan::scan(&partial, &[sql_tag()], &graph, &facts, None);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "mut.ts");
    assert_eq!(findings[0].line, 4);
    assert_eq!(findings[0].target.as_deref(), Some("COLS"));
    assert_eq!(
        findings[0].message,
        "exported SQL fragment `COLS` from `columns.ts` is written, rebound, or escaped"
    );
}
