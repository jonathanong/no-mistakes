use super::{extract_sql_statement_facts, SqlTriggerPeriod};
use crate::codebase::postgres::parse::parse_postgres_sql;
use sqlparser::ast::{
    Function, FunctionArguments, Ident, ObjectName, ObjectNamePart, Query, SetExpr, Statement,
    TableObject, TriggerObject, TriggerObjectKind, TriggerPeriod, Values,
};

fn dummy_fn() -> Function {
    Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("remote"))]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::None,
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    }
}

fn empty_query() -> Query {
    Query {
        with: None,
        body: Box::new(SetExpr::Values(Values {
            explicit_row: false,
            rows: Vec::new(),
            value_keyword: false,
        })),
        order_by: None,
        limit_clause: None,
        fetch: None,
        locks: Vec::new(),
        for_clause: None,
        settings: None,
        format_clause: None,
        pipe_operators: Vec::new(),
    }
}

#[test]
fn insert_and_trigger_from_statement_reject_unrelated_ast() {
    let commit = Statement::Commit {
        chain: false,
        end: false,
        modifier: None,
    };
    assert!(super::insert::from_statement("", &commit, 1).is_none());
    assert!(super::trigger::from_statement("", &commit, 1).is_none());
    let sql = "INSERT INTO items (id) VALUES (1)";
    let Statement::Insert(mut insert) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
        panic!("insert");
    };
    insert.table = TableObject::TableFunction(dummy_fn());
    assert!(super::insert::from_insert(sql, &insert, 1, true)
        .table
        .is_empty());
    insert.table = TableObject::TableQuery(Box::new(empty_query()));
    assert!(super::insert::from_insert(sql, &insert, 1, true)
        .table
        .is_empty());
}

#[test]
fn set_expr_insert_non_insert_and_missing_keyword_lines() {
    let mut inserts = Vec::new();
    let mut insert_n = 0usize;
    super::collect_set_inserts(
        "",
        &SetExpr::Insert(Statement::Commit {
            chain: false,
            end: false,
            modifier: None,
        }),
        &mut insert_n,
        &mut inserts,
    );
    assert!(inserts.is_empty());
    assert_eq!(
        super::lines::nth_keyword_pair_line("SELECT 1", "create", "trigger", 1),
        1
    );
}

#[test]
fn select_collect_explain_copy_and_outer_joins() {
    let sql = "EXPLAIN SELECT id FROM items WHERE id = 1";
    let statement = parse_postgres_sql(sql).unwrap().pop().unwrap();
    let mut selects = Vec::new();
    super::select::collect(sql, &statement, &mut selects);
    assert!(selects
        .iter()
        .any(|select| select.tables.contains(&"items".to_string())));
    let copy = extract_sql_statement_facts("COPY items TO STDOUT");
    assert!(copy.selects.is_empty());
    for join in [
        "SELECT * FROM items LEFT OUTER JOIN accounts ON items.id = accounts.id",
        "SELECT * FROM items RIGHT OUTER JOIN accounts ON items.id = accounts.id",
        "SELECT * FROM items FULL OUTER JOIN accounts ON items.id = accounts.id",
    ] {
        let facts = extract_sql_statement_facts(join);
        assert!(
            facts
                .selects
                .iter()
                .any(|select| !select.predicate_sql.is_empty()),
            "{join} {:#?}",
            facts.selects
        );
    }
}

#[test]
fn trigger_period_for_and_for_row_object() {
    let sql = "CREATE TRIGGER t AFTER INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();";
    let Statement::CreateTrigger(mut trigger) = parse_postgres_sql(sql).unwrap().pop().unwrap()
    else {
        panic!("trigger");
    };
    trigger.period = Some(TriggerPeriod::For);
    trigger.trigger_object = Some(TriggerObjectKind::For(TriggerObject::Row));
    trigger.exec_body = None;
    let fact = super::trigger::from_statement("", &Statement::CreateTrigger(trigger), 1).unwrap();
    assert_eq!(fact.period, SqlTriggerPeriod::Other);
    assert!(fact.for_each_row);
    assert!(fact.function.is_empty());
}

#[test]
fn insert_source_without_query_or_rows_is_unstable() {
    let sql = "INSERT INTO items (id, seen) VALUES (1, 'a')";
    let Statement::Insert(mut insert) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
        panic!("insert");
    };
    insert.source = None;
    assert!(super::insert::from_insert(sql, &insert, 1, true)
        .assignments
        .is_empty());
    insert.source = Some(Box::new(empty_query()));
    assert!(
        super::insert::from_insert(sql, &insert, 1, true)
            .assignments
            .iter()
            .all(|assignment| assignment.form == super::SqlValueForm::Other),
        "{:#?}",
        super::insert::from_insert(sql, &insert, 1, true).assignments
    );
}

#[test]
fn insert_set_assignments_are_kept() {
    let sql = "INSERT INTO items (id, seen) VALUES (1, 'a')";
    let Statement::Insert(mut insert) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
        panic!("insert");
    };
    let Statement::Update(update) = parse_postgres_sql("UPDATE items SET seen = now()")
        .unwrap()
        .pop()
        .unwrap()
    else {
        panic!("update");
    };
    insert.assignments = update.assignments;
    assert!(
        super::insert::from_insert(sql, &insert, 1, true)
            .assignments
            .iter()
            .any(|assignment| assignment.column == "seen"
                && matches!(assignment.form, super::SqlValueForm::Volatile { .. })),
        "{:#?}",
        super::insert::from_insert(sql, &insert, 1, true).assignments
    );
}

#[test]
fn overriding_user_value_drops_source_forms() {
    let sql = "INSERT INTO items (id, seen) VALUES (1, 'a')";
    let Statement::Insert(insert) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
        panic!("insert");
    };
    assert!(super::insert::from_insert(
        "INSERT INTO items (id, seen) OVERRIDING /* skip */
USER VALUE VALUES (1, 'a')",
        &insert,
        1,
        true,
    )
    .assignments
    .is_empty());
}
