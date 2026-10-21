use super::*;
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use std::path::PathBuf;

fn caller_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-required-predicates/variants/builder-text/src/caller.mts",
    )
}

fn caller() -> super::super::EmbeddedSqlFileFacts {
    extract_from_path(&caller_path())
}

fn function_lines(source: &str, name: &str) -> std::ops::Range<u32> {
    let marker = format!("function {name}(");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing function {name}"));
    let line = source[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
        + 1;
    let after = start + marker.len();
    let end = source[after..]
        .find("\nexport async function ")
        .map(|index| {
            line + source[start..after + index]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count() as u32
        })
        .unwrap_or(u32::MAX);
    line..end
}

fn call_in_function<'a>(
    facts: &'a super::super::EmbeddedSqlFileFacts,
    source: &str,
    name: &str,
) -> &'a crate::codebase::postgres::embedded::EmbeddedSqlCall {
    let lines = function_lines(source, name);
    let mut matches = facts.calls.iter().filter(|call| lines.contains(&call.line));
    let call = matches
        .next()
        .unwrap_or_else(|| panic!("{name} has no executor call"));
    assert!(
        matches.next().is_none(),
        "{name} must have one executor call so the lookup cannot follow a shifted line"
    );
    call
}

#[test]
fn builder_text_executor_arguments_recover_composed_sql_and_fail_closed() {
    let source = std::fs::read_to_string(caller_path()).expect("caller.mts");
    let facts = caller();
    let straight = call_in_function(&facts, &source, "straight");
    assert_eq!(straight.kind, EmbeddedSqlKind::Composed);
    assert_eq!(straight.variants.len(), 0);
    let sql = straight.sql_text.as_deref().expect("recovered SQL");
    assert!(sql.contains("WITH picked AS"), "{sql}");
    assert!(sql.contains("view_active_plans"), "{sql}");
    assert!(sql.contains("sql_placeholder_"), "{sql}");

    let branching = call_in_function(&facts, &source, "branching");
    assert_eq!(branching.kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(branching.variants.len(), 2, "{branching:#?}");
    assert!(
        branching
            .variants
            .iter()
            .all(|variant| variant.sql_text.contains("WITH picked AS")),
        "{branching:#?}"
    );

    for name in [
        "unknownBuilder",
        "extraInterpolation",
        "foreignValues",
        "notABuilder",
        "partialBuilder",
        "missingValues",
        "extraArgument",
        "textUsedTwice",
        "computedMember",
        "wrappedValues",
        "insideLoop",
    ] {
        let call = call_in_function(&facts, &source, name);
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{name}");
        assert!(
            call.variants.is_empty(),
            "{name} must not recover builder.text alternatives: {call:#?}"
        );
        assert_ne!(
            call.kind,
            EmbeddedSqlKind::Composed,
            "{name} must not take the builder.text composed path: {call:#?}"
        );
    }
}
