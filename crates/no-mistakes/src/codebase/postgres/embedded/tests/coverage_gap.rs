use super::super::{
    extract_embedded_sql_from_program_with_projections, helpers::splice::splice, EmbeddedSqlCall,
    EmbeddedSqlFileFacts, EmbeddedSqlOptions, EmbeddedSqlVariant, SqlByteOrigins,
};
use crate::codebase::postgres::embedded::site_identity::Invocations;
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
    let seen = |name: &str| {
        calls_in(&source, &facts, name)
            .into_iter()
            .map(|call| {
                format!(
                    "{}{}",
                    if call.is_unanalyzable() {
                        "unanalyzable "
                    } else {
                        ""
                    },
                    sqls(call).into_iter().collect::<Vec<_>>().join("|")
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    let mut problems = Vec::new();
    for (name, expected) in [
        ("numericHole", "SELECT 1"),
        ("nestedFlag", "SELECT a|SELECT b|SELECT c|SELECT d"),
        ("constrainedPiece", "SELECT a|SELECT b|SELECT c"),
        ("plainHole", "SELECT orders"),
        ("literalRead", "SELECT 1"),
        ("stringKey", "SELECT orders"),
        ("guardedTables", "SELECT accounts|SELECT orders"),
    ] {
        let actual = seen(name);
        if actual != expected {
            problems.push(format!("{name}: {actual}"));
        }
    }
    let null_byte = seen("nullByte");
    if !null_byte.contains('\0') {
        problems.push(format!("nullByte: {null_byte}"));
    }
    let newline = seen("escapedNewline");
    if !newline.contains('\n') {
        problems.push(format!("escapedNewline: {newline}"));
    }
    let marked = seen("markedHole");
    if !marked.contains('\u{10FFFE}') {
        problems.push(format!("markedHole: {marked}"));
    }
    for name in [
        "mutatingTest",
        "spreadBuilderValues",
        "numericKey",
        "deepTemplate",
    ] {
        let actual = seen(name);
        if !actual.starts_with("unanalyzable") {
            problems.push(format!("{name}: {actual}"));
        }
    }
    for name in [
        "spreadConstructor",
        "emptyElement",
        "missingHasArgument",
        "spreadHasArgument",
        "bareAppend",
        "spreadAppend",
    ] {
        let actual = seen(name);
        if actual.contains("orders") {
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

fn variant(sql: &str) -> EmbeddedSqlVariant {
    EmbeddedSqlVariant {
        sql_text: sql.to_string(),
        sql_source_offsets: SqlByteOrigins::from(vec![0; sql.len()]),
        invocations: Invocations::default(),
        line: 1,
        sql_source_positions: Vec::new(),
        recovered_placeholder_positions: Vec::new(),
        append_sites: Vec::new(),
        first_append_offset: None,
        variants_truncated: false,
    }
}

#[test]
fn coverage_gap_splice_abstains_on_bad_origins_and_too_many_alternatives() {
    let holes = crate::fx::FxHashMap::default();
    let kept = crate::codebase::postgres::EmbeddedSqlCall {
        variants: vec![variant("SELECT 1")],
        ..Default::default()
    };
    let spliced = splice(&kept, &holes).expect("one alternative");
    assert_eq!(spliced.variants[0].sql_text, "SELECT 1");

    let mut drifted = variant("SELECT 1");
    drifted.sql_source_offsets = SqlByteOrigins::default();
    assert!(splice(
        &crate::codebase::postgres::EmbeddedSqlCall {
            variants: vec![drifted],
            ..Default::default()
        },
        &holes,
    )
    .is_none());

    let mut unmarked = variant("SELECT 1");
    unmarked.recovered_placeholder_positions = vec![(1, 1)];
    assert!(splice(
        &crate::codebase::postgres::EmbeddedSqlCall {
            variants: vec![unmarked],
            ..Default::default()
        },
        &holes,
    )
    .is_none());

    let mut invoked = variant("SELECT 1");
    invoked.recovered_placeholder_positions = vec![(1, 1)];
    invoked.invocations.record(1, 0);
    assert!(splice(
        &crate::codebase::postgres::EmbeddedSqlCall {
            variants: vec![invoked],
            ..Default::default()
        },
        &holes,
    )
    .is_none());

    let mut past = variant("SELECT 1");
    past.first_append_offset = Some(100);
    let spliced = splice(
        &crate::codebase::postgres::EmbeddedSqlCall {
            variants: vec![past],
            ..Default::default()
        },
        &holes,
    )
    .expect("an append offset past the text does not drop the alternative");
    assert_eq!(spliced.variants[0].sql_text, "SELECT 1");

    let many = crate::codebase::postgres::EmbeddedSqlCall {
        variants: (0..17)
            .map(|index| variant(&format!("SELECT {index}")))
            .collect(),
        ..Default::default()
    };
    assert!(splice(&many, &holes).is_none(), "17 alternatives abstain");

    let mut mirror = crate::codebase::postgres::EmbeddedSqlCall {
        sql_text: Some("SELECT 1".into()),
        kind: crate::codebase::postgres::EmbeddedSqlKind::Inline,
        line: 1,
        sql_source_offsets: SqlByteOrigins::from(vec![1]),
        ..Default::default()
    };
    assert!(splice(&mirror, &holes).is_none());
    mirror.sql_source_offsets = SqlByteOrigins::default();
    let spliced = splice(&mirror, &holes).expect("no hole");
    assert_eq!(spliced.sql_text.as_deref(), Some("SELECT 1"));
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
