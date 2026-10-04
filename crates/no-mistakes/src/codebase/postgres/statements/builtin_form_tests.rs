use super::{extract_sql_statement_facts, SqlValueForm};
use crate::codebase::postgres::parse::parse_postgres_sql;
use sqlparser::ast::Statement;

#[test]
fn national_now_qualified_coalesce_quoted_placeholder_and_scoped_overriding() {
    let national = extract_sql_statement_facts("INSERT INTO items (id, seen) VALUES (1, N'now')");
    assert!(matches!(
        national.inserts[0].assignments[1].form,
        SqlValueForm::Other
    ));
    let epoch = extract_sql_statement_facts("INSERT INTO items (id, seen) VALUES (1, 'epoch')");
    assert!(matches!(
        epoch.inserts[0].assignments[1].form,
        SqlValueForm::Literal
    ));
    let qualified = extract_sql_statement_facts(
        "INSERT INTO items (id, seen) VALUES (1, pg_catalog.coalesce('a'))",
    );
    assert!(matches!(
        qualified.inserts[0].assignments[1].form,
        SqlValueForm::Other
    ));
    let quoted = extract_sql_statement_facts(
        "INSERT INTO items (id, seen) VALUES (1, \"sql_placeholder_1\")",
    );
    assert!(matches!(
        quoted.inserts[0].assignments[1].form,
        SqlValueForm::SelfRef { .. }
    ));
    let signed = extract_sql_statement_facts(
        "INSERT INTO items (id, n) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET n = -$1;",
    );
    assert!(matches!(
        signed.inserts[0].on_conflict.as_ref().unwrap().assignments[0].form,
        SqlValueForm::Placeholder
    ));
    let first_sql = "INSERT INTO items (id, seen) VALUES (1, 'a')";
    let second_sql = "INSERT INTO items (id, seen) VALUES (2, 'b')";
    let combined = "INSERT INTO items (id, seen) VALUES (1, 'a'); INSERT INTO items (id, seen) OVERRIDING USER VALUE VALUES (2, 'b');";
    let Statement::Insert(first) = parse_postgres_sql(first_sql).unwrap().pop().unwrap() else {
        panic!("first");
    };
    let Statement::Insert(second) = parse_postgres_sql(second_sql).unwrap().pop().unwrap() else {
        panic!("second");
    };
    assert!(!super::insert::from_insert_prepared(
        &super::lines::InsertSources::new(combined),
        &first,
        1,
        false,
        None
    )
    .assignments
    .is_empty());
    assert!(super::insert::from_insert_prepared(
        &super::lines::InsertSources::new(combined),
        &second,
        2,
        false,
        None
    )
    .assignments
    .is_empty());
}

#[test]
fn prepared_insert_ranges_preserve_quoted_decoys_and_scoped_overriding() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres/source-positions/fixture/inserts.sql"
    ));
    // OVERRIDING is intentionally outside the parser's grammar. Apply the same
    // parsed INSERT to each prepared range to test its source-only semantics.
    let statements = parse_postgres_sql(sql.lines().nth(2).unwrap()).unwrap();
    let Statement::Insert(insert) = &statements[0] else {
        panic!("fixture INSERT")
    };
    let sources = super::lines::InsertSources::new(sql);
    let facts = (1..=3)
        .map(|n| super::insert::from_insert_prepared(&sources, insert, n, true, None))
        .collect::<Vec<_>>();
    assert_eq!(
        facts.iter().map(|fact| fact.line).collect::<Vec<_>>(),
        vec![3, 4, 5]
    );
    assert_eq!(
        facts
            .iter()
            .map(|fact| fact.assignments.len())
            .collect::<Vec<_>>(),
        vec![1, 0, 1]
    );
}
