use super::super::{
    extract_embedded_sql_from_program_with_projections, EmbeddedSqlCall, EmbeddedSqlFileFacts,
    EmbeddedSqlOptions,
};
use std::collections::BTreeSet;

fn options() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        crate::codebase::postgres::embedded::TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ])
}

fn source(name: &str) -> (std::path::PathBuf, String) {
    (super::fixture(name), super::read_fixture(name))
}

fn extract(name: &str) -> (String, EmbeddedSqlFileFacts) {
    let (path, source) = source(name);
    let facts = super::super::extract_embedded_sql_from_source(&path, &source, &options());
    (source, facts)
}

fn project(name: &str, exact: bool) -> (String, EmbeddedSqlFileFacts) {
    let (path, text) = source(name);
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    let facts = crate::ast::with_program(&path, &text, |program, source| {
        extract_embedded_sql_from_program_with_projections(
            &path,
            program,
            source,
            &options,
            (false, exact),
        )
    })
    .expect("parse");
    (text, facts)
}

fn calls_in<'a>(
    source: &str,
    facts: &'a EmbeddedSqlFileFacts,
    name: &str,
) -> Vec<&'a EmbeddedSqlCall> {
    let marker = format!("function {name}");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {name}"));
    let line_of = |index: usize| {
        source[..index]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count() as u32
            + 1
    };
    let start_line = line_of(start);
    let after = start + marker.len();
    let end = source[after..]
        .find("\nexport ")
        .map(|offset| after + offset)
        .unwrap_or(source.len());
    let end_line = line_of(end);
    facts
        .calls
        .iter()
        .filter(|call| (start_line..end_line).contains(&call.line))
        .collect()
}

fn seen_call(call: &EmbeddedSqlCall) -> String {
    format!(
        "{}{}",
        if call.is_unanalyzable() {
            "unanalyzable "
        } else {
            ""
        },
        sqls(call).into_iter().collect::<Vec<_>>().join("|")
    )
}

fn sqls(call: &EmbeddedSqlCall) -> BTreeSet<String> {
    if call.variants.is_empty() {
        call.sql_text.iter().cloned().collect()
    } else {
        call.variants
            .iter()
            .map(|variant| variant.sql_text.clone())
            .collect()
    }
}

fn only<'a>(source: &str, facts: &'a EmbeddedSqlFileFacts, name: &str) -> &'a EmbeddedSqlCall {
    let calls = calls_in(source, facts, name);
    assert_eq!(calls.len(), 1, "{name}: {calls:#?}");
    calls[0]
}

#[test]
fn coverage_gap_clauses_guards_and_literals() {
    let (source, facts) = extract("coverage-gap.ts");
    let mut problems = Vec::new();
    for (name, expected) in [
        ("numericHole", "n=1 SELECT 1"),
        ("nestedFlag", "n=1 SELECT a|SELECT b|SELECT c|SELECT d"),
        ("constrainedPiece", "n=1 SELECT a|SELECT b|SELECT c"),
        ("plainHole", "n=1 SELECT orders"),
        ("literalRead", "n=1 SELECT 1"),
        ("stringKey", "n=1 SELECT orders"),
        ("guardedTables", "n=1 SELECT accounts|SELECT orders"),
        ("nullByte", "n=1 SELECT \0 a"),
        ("escapedNewline", "n=1 SELECT \n"),
        ("markedHole", "n=1 SELECT \u{10FFFE}"),
        ("mutatingTest", "n=1 unanalyzable SELECT sql_placeholder_1"),
        ("spreadBuilderValues", "n=1 unanalyzable sql_placeholder_1"),
        ("numericKey", "n=1 unanalyzable SELECT sql_placeholder_1"),
        ("deepTemplate", "n=1 unanalyzable SELECT sql_placeholder_1"),
        ("spreadConstructor", "n=1 unanalyzable SELECT "),
        ("emptyElement", "n=1 unanalyzable SELECT "),
        ("missingHasArgument", "n=1 unanalyzable SELECT "),
        ("spreadHasArgument", "n=1 unanalyzable SELECT "),
        ("bareAppend", "n=1 unanalyzable SELECT 1"),
        ("spreadAppend", "n=1 unanalyzable SELECT 1"),
    ] {
        let calls = calls_in(&source, &facts, name);
        let body = calls
            .iter()
            .map(|call| seen_call(call))
            .collect::<Vec<_>>()
            .join("; ");
        let actual = format!("n={} {body}", calls.len());
        if actual != expected {
            problems.push(format!("{name}: {actual}"));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn coverage_gap_exact_origins_fail_closed() {
    let (source, on) = project("coverage-gap-origins.ts", true);
    let (_, off) = project("coverage-gap-origins.ts", false);
    let literal = only(&source, &on, "literalQuery");
    assert_eq!(literal.sql_text.as_deref(), Some("SELECT 2"));
    assert!(!literal.sql_source_offsets.is_empty());
    assert!(only(&source, &off, "literalQuery")
        .sql_source_offsets
        .is_empty());
    let missing = only(&source, &on, "missingBinding");
    assert!(missing.sql_text.is_none() || missing.is_unanalyzable());
    assert!(missing.sql_source_offsets.is_empty());
    let identified = only(&source, &on, "throughIdentifier");
    assert_eq!(identified.sql_text.as_deref(), Some("SELECT 1"));
    assert!(identified.sql_source_offsets.is_empty());
    let broken = only(&source, &on, "brokenHole");
    assert!(broken.is_unanalyzable() || broken.sql_source_offsets.is_empty());
}

#[test]
fn coverage_gap_transaction_arms_and_lock_helpers() {
    let (path, text) = source("coverage-gap-transactions.ts");
    let mut options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_scoped_executors(&["beginTransaction".into()], &[]);
    options.track_transaction_helpers = true;
    let facts = super::super::extract_embedded_sql_from_source(&path, &text, &options);
    let arm = |name: &str| calls_in(&text, &facts, name);
    let taken_true = arm("takenTrue");
    assert!(
        taken_true.iter().any(|call| call
            .sql_text
            .as_deref()
            .is_some_and(|sql| sql.contains("SELECT 1"))),
        "{taken_true:#?}"
    );
    assert!(
        taken_true.iter().all(|call| call.transaction_id.is_some()),
        "{taken_true:#?}"
    );
    let taken_false = arm("takenFalse");
    assert!(
        taken_false.iter().any(|call| call
            .sql_text
            .as_deref()
            .is_some_and(|sql| sql.contains("SELECT 2"))),
        "{taken_false:#?}"
    );
    assert!(
        taken_false.iter().all(|call| call.transaction_id.is_some()),
        "{taken_false:#?}"
    );
    let thrown = arm("unknownThrow");
    assert!(!thrown.is_empty(), "{thrown:#?}");
    assert!(thrown.iter().all(|call| call.transaction_id.is_some()));
    let else_thrown = arm("unknownElseThrow");
    assert!(!else_thrown.is_empty(), "{else_thrown:#?}");
    assert!(else_thrown.iter().all(|call| call.transaction_id.is_some()));
    let joined = arm("unknownJoin");
    assert!(joined.len() >= 2, "{joined:#?}");
    assert!(joined.iter().all(|call| call.transaction_id.is_some()));
    assert_eq!(joined[0].transaction_id, joined[1].transaction_id);
    let helpers: BTreeSet<_> = facts.lock_helper_summaries.keys().cloned().collect();
    assert!(helpers.contains("goodLock"), "{helpers:?}");
    assert!(helpers.contains("callableLock"), "{helpers:?}");
    assert!(!helpers.contains("missingSql"), "{helpers:?}");
    assert!(!helpers.contains("missingArgument"), "{helpers:?}");
}

#[test]
fn coverage_gap_each_rows_and_parameter_appends() {
    let (_, facts) = extract("coverage-gap-each.ts");
    let published: BTreeSet<_> = facts.calls.iter().flat_map(sqls).collect();
    assert!(
        published.contains("SELECT orders") && published.contains("SELECT accounts"),
        "finite rows publish both mapped statements: {published:?}"
    );
    assert!(facts.calls.len() >= 4, "{:?}", facts.calls);

    let path = super::fixture("coverage-gap-builders.ts");
    let source = super::read_fixture("coverage-gap-builders.ts");
    let facts = super::super::extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    let intrinsic = extract("coverage-gap-intrinsics.ts").1;
    assert!(
        !intrinsic.calls.is_empty(),
        "intrinsic declarations must still leave the query call"
    );
    assert!(
        facts.calls.len() >= 3,
        "each write is visited: {:?}",
        facts.calls.len()
    );
    assert!(
        facts
            .calls
            .iter()
            .any(|call| call.sql_text.is_none() || call.is_unanalyzable()),
        "an append without a fragment abstains: {:#?}",
        facts.calls
    );
}
