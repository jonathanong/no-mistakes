use super::super::body::decode;
use super::super::procedural_omit::label::opening_labels;
use super::super::procedural_omit::non_sql_marks;
use super::super::{
    PostgresSqlPosition, PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind,
    PostgresSqlSpan,
};
use super::procedural_occurrences::{block, kinds};
use sqlparser::ast::DollarQuotedString;
use sqlparser::tokenizer::Token;

use PostgresSqlProceduralOccurrenceKind::{ControlFlow, Utility};

fn at(offset: usize) -> PostgresSqlPosition {
    PostgresSqlPosition {
        offset,
        line: 1,
        column: offset + 1,
    }
}

fn occurrence(
    kind: PostgresSqlProceduralOccurrenceKind,
    start: usize,
    end: usize,
) -> PostgresSqlProceduralOccurrence {
    PostgresSqlProceduralOccurrence {
        kind,
        span: PostgresSqlSpan {
            start: at(start),
            end: at(end),
        },
        occurrences: Vec::new(),
    }
}

fn whole(len: usize) -> PostgresSqlSpan {
    PostgresSqlSpan {
        start: at(0),
        end: at(len),
    }
}

fn dollar(source: &str) -> super::super::body::Body<'_> {
    let sql = source
        .strip_prefix("$$")
        .and_then(|rest| rest.strip_suffix("$$"))
        .unwrap();
    decode(
        &Token::DollarQuotedString(DollarQuotedString {
            value: sql.to_string(),
            tag: None,
        }),
        &whole(source.len()),
        source,
    )
    .unwrap()
}

fn labels_at(
    source: &str,
    keyword: &str,
    kind: PostgresSqlProceduralOccurrenceKind,
) -> Vec<(usize, usize)> {
    let body = dollar(source);
    let at = body.sql.find(keyword).unwrap();
    opening_labels(
        &body,
        &[occurrence(
            kind,
            body.offset(at).unwrap(),
            body.offset(at + keyword.len()).unwrap(),
        )],
    )
}

fn label_range(source: &str, label: &str) -> (usize, usize) {
    let start = source.find(label).unwrap();
    (start, start + label.len())
}

#[test]
fn labeled_loop_keeps_create_and_drops_the_opening_label() {
    // The public span starts at FOR. The label is omitted with that interval.
    let (sql, parsed) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); <<l>> FOR i IN 1..0 LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
    );
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Utility", "ControlFlow[\"ControlFlow\"]"]
    );
    assert_eq!(parsed.occurrences.len(), 2);
    let span = &sql[parsed.occurrences[1].span.start.offset..parsed.occurrences[1].span.end.offset];
    assert!(span.contains("END LOOP"), "{span}");
    assert!(span.contains("FOR"), "{span}");
    assert!(!span.contains("<<l>>"), "{span}");
    assert_eq!(parsed.statements.len(), 1);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(!parsed.statements[0].sql.contains("<<l>>"));
}

#[test]
fn unlabeled_raise_between_creates_stays_complete() {
    let (_, parsed) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); RAISE NOTICE 'x'; CREATE TABLE b(id int); END $$;",
    );
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.statements.len(), 2);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(parsed.statements[1].sql.contains("CREATE TABLE b"));
}

#[test]
fn loop_with_dml_or_unknown_sql_beside_create_stays_incomplete() {
    let (_, insert) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); <<l>> FOR i IN 1..2 LOOP INSERT INTO a VALUES (i); END LOOP; END $$;",
    );
    assert!(!insert.complete);
    assert_eq!(insert.statements.len(), 1);
    assert!(insert.statements[0].sql.contains("CREATE TABLE a"));
    assert!(!insert.statements[0].sql.contains("<<l>>"));
    assert!(!insert.statements[0].sql.contains("INSERT"));
    assert!(insert.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Static DML is a source occurrence, not an executed statement")
    }));

    let (_, unknown) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); <<l>> FOR i IN 1..0 LOOP NULL; END LOOP; END $$;",
    );
    assert!(!unknown.complete);
    assert_eq!(unknown.statements.len(), 1);
    assert!(unknown.statements[0].sql.contains("CREATE TABLE a"));
    assert!(!unknown.statements[0].sql.contains("<<l>>"));
    assert!(unknown.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Unsupported procedural occurrence")
    }));
}

#[test]
fn empty_and_quoted_opening_labels_are_omitted_with_the_loop() {
    let (_, empty) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); <<>> FOR i IN 1..0 LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
    );
    assert!(!empty.complete, "{:?}", empty.diagnostics);
    assert!(empty.statements.iter().any(|statement| {
        statement.sql.contains("CREATE TABLE a") && !statement.sql.contains("<<>>")
    }));
    assert!(empty.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Unsupported procedural occurrence")
    }));

    let (_, quoted) = block(
        r#"DO $$ BEGIN CREATE TABLE a(id int); <<"l">> FOR i IN 1..0 LOOP RAISE NOTICE 'x'; END LOOP; END $$;"#,
    );
    assert!(quoted.complete, "{:?}", quoted.diagnostics);
    assert!(quoted.statements[0].sql.contains("CREATE TABLE a"));
    assert!(!quoted.statements[0].sql.contains("<<"));
}

#[test]
fn label_before_a_sql_statement_is_not_omitted() {
    // A utility span does not include the label, and the label is not omitted.
    let (sql, parsed) = block("DO $$ BEGIN <<l>> CREATE TABLE a(id int); END $$;");
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.statements.is_empty(), "{:?}", parsed.statements);
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    let span = &sql[parsed.occurrences[0].span.start.offset..parsed.occurrences[0].span.end.offset];
    assert!(span.contains("CREATE TABLE a"), "{span}");
    assert!(!span.contains("<<l>>"), "{span}");
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("<<")));
}

#[test]
fn escaped_body_maps_the_label_back_to_original_offsets() {
    let (_, parsed) = block(
        "DO 'BEGIN RAISE NOTICE ''z''; CREATE TABLE a(id int); <<l>> FOR i IN 1..0 LOOP RAISE NOTICE ''x''; END LOOP; END';",
    );
    assert!(
        parsed.statements.iter().any(|statement| {
            statement.sql.contains("CREATE TABLE a") && !statement.sql.contains("<<l>>")
        }),
        "{:?}",
        parsed.statements
    );
    let source = "'BEGIN '' <<l>> FOR'";
    let decoded = "BEGIN ' <<l>> FOR";
    let body = decode(
        &Token::SingleQuotedString(decoded.to_string()),
        &whole(source.len()),
        source,
    )
    .unwrap();
    let at = decoded.find("FOR").unwrap();
    let found = opening_labels(
        &body,
        &[occurrence(
            ControlFlow,
            body.offset(at).unwrap(),
            body.offset(at + 3).unwrap(),
        )],
    );
    assert_eq!(found, vec![label_range(source, "<<l>>")]);
}

#[test]
fn opening_label_forms_match_the_walker_and_skip_sql() {
    assert_eq!(
        labels_at("$$<<l>> FOR$$", "FOR", ControlFlow),
        vec![label_range("$$<<l>> FOR$$", "<<l>>")]
    );
    assert_eq!(
        labels_at("$$<<l>>FOR$$", "FOR", ControlFlow),
        vec![label_range("$$<<l>>FOR$$", "<<l>>")]
    );
    assert_eq!(
        labels_at("$$<< l >>\nFOR$$", "FOR", ControlFlow),
        vec![label_range("$$<< l >>\nFOR$$", "<< l >>")]
    );
    assert_eq!(
        labels_at("$$<<>> FOR$$", "FOR", ControlFlow),
        vec![label_range("$$<<>> FOR$$", "<<>>")]
    );
    assert_eq!(
        labels_at(r#"$$<<"Retry">> FOR$$"#, "FOR", ControlFlow),
        vec![label_range(r#"$$<<"Retry">> FOR$$"#, r#"<<"Retry">>"#)]
    );
    assert_eq!(
        labels_at(r#"$$<<"a""b">> FOR$$"#, "FOR", ControlFlow),
        vec![label_range(r#"$$<<"a""b">> FOR$$"#, r#"<<"a""b">>"#)]
    );
    assert!(labels_at("$$<<l>> FOR$$", "FOR", Utility).is_empty());
    assert!(labels_at("$$<<foo bar>> FOR$$", "FOR", ControlFlow).is_empty());
    assert!(labels_at("$$<<1>> FOR$$", "FOR", ControlFlow).is_empty());
    assert!(labels_at("$$<< >> FOR$$", "FOR", ControlFlow).is_empty());
    assert!(labels_at("$$AB FOR$$", "FOR", ControlFlow).is_empty());
    assert!(labels_at("$$+<<l>> FOR$$", "FOR", ControlFlow).is_empty());
    assert!(labels_at("$$<<<>> FOR$$", "FOR", ControlFlow).is_empty());
    assert!(labels_at(r#"$$foo" >> FOR$$"#, "FOR", ControlFlow).is_empty());

    let glued = "$$<<>>+$$";
    let body = dollar(glued);
    let plus = body.sql.find('+').unwrap();
    assert!(opening_labels(
        &body,
        &[occurrence(
            ControlFlow,
            body.offset(plus).unwrap(),
            body.offset(plus + 1).unwrap(),
        )],
    )
    .is_empty());

    let prefixed = "$$+<<>>$$";
    let body = dollar(prefixed);
    let end = body.offset(body.sql.len()).unwrap();
    assert!(opening_labels(&body, &[occurrence(ControlFlow, end, end)]).is_empty());

    let inside = "$$\u{e9}FOR$$";
    let body = dollar(inside);
    let mid = body.offset(1).unwrap();
    assert!(opening_labels(&body, &[occurrence(ControlFlow, mid, mid + 1)]).is_empty());

    let source = "$$<<l>> FOR$$";
    let body = dollar(source);
    let at = body.sql.find("FOR").unwrap();
    let start = body.offset(at).unwrap();
    let mut loop_occ = occurrence(ControlFlow, start, body.offset(at + 3).unwrap());
    let (label_start, label_end) = label_range(source, "<<l>>");
    loop_occ
        .occurrences
        .push(occurrence(Utility, label_start, label_end));
    assert!(opening_labels(&body, &[loop_occ]).is_empty());
    assert!(opening_labels(&body, &[occurrence(ControlFlow, 0, 1)]).is_empty());
}

#[test]
fn cover_adds_a_label_interval_without_an_extra_pass() {
    let mut marks = non_sql_marks(4, 10, &[occurrence(ControlFlow, 8, 12)]);
    assert_eq!(marks.span_passes(), 1);
    assert_eq!(marks.body_passes(), 1);
    marks.cover(8, 8);
    marks.cover(0, 3);
    marks.cover(13, 15);
    assert!(marks.overlaps(8, 9));
    assert!(!marks.overlaps(4, 6));
    marks.cover(6, 9);
    assert!(marks.overlaps(6, 7));
    marks.cover(12, 14);
    assert!(marks.overlaps(13, 14));
    assert!(!marks.overlaps(4, 6));
    assert_eq!(marks.span_passes(), 1);
    assert_eq!(marks.body_passes(), 1);
}
