use super::super::{AnnotationHeads, Span};
use super::collect_with_helpers;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan, TsFileFacts,
};
use std::path::PathBuf;
pub(super) fn fixture(name: &str) -> (String, TsFileFacts, AnnotationHeads) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let (ts, heads) = crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                imports: true,
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        assert!(ts.module_bindings.is_some());
        let heads = collect_with_helpers(
            program,
            &ts,
            executors(&path, &source, program).into_iter(),
            &Default::default(),
            true,
            false,
        );
        (ts, heads)
    })
    .unwrap();
    (source, ts, heads)
}
pub(super) fn recollect(name: &str, ts: &TsFileFacts) -> AnnotationHeads {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        collect_with_helpers(
            program,
            ts,
            executors(&path, &source, program).into_iter(),
            &Default::default(),
            true,
            false,
        )
    })
    .unwrap()
}
pub(super) fn decisions(source: &str, heads: &AnnotationHeads) -> Vec<(Span, bool)> {
    heads
        .calls
        .iter()
        .filter(|((start, _), _)| {
            let tail = &source[*start as usize..];
            tail.starts_with("read(") || tail.starts_with("read?.(")
        })
        .map(|(span, _)| (*span, heads.accepts(*span)))
        .collect()
}
pub(super) fn assert_markers(source: &str, heads: &AnnotationHeads) -> usize {
    let calls = decisions(source, heads);
    let mut cursor = 0;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        if let Some((before, expected)) = line.split_once("// low: ") {
            let end = offset + before.len();
            let batch = calls[cursor..]
                .iter()
                .take_while(|(span, _)| span.1 as usize <= end)
                .map(|(_, accepted)| *accepted)
                .collect::<Vec<_>>();
            let expected = expected
                .trim()
                .split(',')
                .map(|value| value.parse::<bool>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(batch, expected, "{line}");
            cursor += batch.len();
        }
        offset += line.len();
    }
    assert_eq!(
        cursor,
        calls.len(),
        "every saved read call has an explicit oracle"
    );
    cursor
}

fn executors(
    path: &std::path::Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
) -> Vec<Span> {
    crate::codebase::postgres::extract_embedded_sql_from_program(
        path,
        program,
        source,
        &EmbeddedSqlOptions::configured("@bench/db", &[]),
    )
    .call_spans
}
