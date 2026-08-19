use super::collect_fetch_calls_in_file;
use crate::ast;
use crate::fetch::types::FetchOccurrence;
use oxc_span::Span;
use std::path::PathBuf;

fn collect_fetch_calls(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    rel_file: &str,
    span: Span,
) -> Vec<FetchOccurrence> {
    collect_fetch_calls_in_file(program, source, rel_file)
        .into_iter()
        .filter(|(call_span, _)| call_span.start >= span.start && call_span.end <= span.end)
        .map(|(_, fetch)| fetch)
        .collect()
}

fn fixture(category: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(name)
        .join("fixture")
}

fn check(fixture_path: &std::path::Path) -> usize {
    let source = std::fs::read_to_string(fixture_path).expect("fixture file must be readable");
    let span = oxc_span::Span::new(0, source.len() as u32);
    ast::with_program(fixture_path, &source, |program, _| {
        collect_fetch_calls(
            program,
            &source,
            fixture_path.to_str().unwrap_or("test.tsx"),
            span,
        )
        .len()
    })
    .unwrap()
}

#[test]
fn detects_fetch_call() {
    let path = fixture("react-traits-fetch", "detect-fetch").join("test.tsx");
    assert_eq!(check(&path), 1);
}

#[test]
fn no_fetch() {
    let path = fixture("react-traits-fetch", "no-fetch").join("test.tsx");
    assert_eq!(check(&path), 0);
}
