use super::super::EmbeddedSqlKind;
use super::*;

pub(super) struct Site<'a> {
    pub(super) line: u32,
    pub(super) expression: &'a str,
    pub(super) sql: Option<&'a str>,
    pub(super) kind: EmbeddedSqlKind,
}

pub(super) fn scoped_options() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["openTransaction".into()], &["TxExecutor".into()])
}

pub(super) fn assert_sites(name: &str, options: &EmbeddedSqlOptions, expected: &[Site<'_>]) {
    let source = read_fixture(name);
    let facts = extract_embedded_sql_from_source(&fixture(name), &source, options);
    assert_eq!(facts.calls.len(), expected.len(), "{name}: {facts:?}");
    assert_eq!(
        facts.call_spans.len(),
        expected.len(),
        "{name}: missing spans"
    );
    let mut sites = facts
        .calls
        .iter()
        .zip(&facts.call_spans)
        .collect::<Vec<_>>();
    sites.sort_by_key(|(_, span)| span.0);
    for ((call, &(start, end)), expected) in sites.into_iter().zip(expected) {
        assert_eq!(
            source.get(start as usize..end as usize),
            Some(expected.expression),
            "{name}: wrong call identity"
        );
        assert_eq!(call.line, expected.line, "{name}: {}", expected.expression);
        assert_eq!(
            call.sql_text.as_deref(),
            expected.sql,
            "{name}: {}",
            expected.expression
        );
        assert_eq!(call.kind, expected.kind, "{name}: {}", expected.expression);
        if expected.kind == EmbeddedSqlKind::Dynamic {
            assert!(call.variants.is_empty() && call.is_unanalyzable());
        } else {
            assert!(!call.is_unanalyzable());
        }
    }
}

mod aliases;
mod diagnostics;
mod receivers;
