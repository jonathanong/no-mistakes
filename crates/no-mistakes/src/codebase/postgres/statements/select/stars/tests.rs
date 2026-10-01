use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn stars(sql: &str) -> Vec<(String, bool, Option<String>, usize)> {
    extract_sql_statement_facts(sql)
        .selects
        .into_iter()
        .flat_map(|select| select.star_projections)
        .map(|star| {
            (
                star.relation,
                star.qualified,
                star.within_function,
                star.line,
            )
        })
        .collect()
}

fn returning(sql: &str) -> Vec<(String, bool, Option<String>)> {
    extract_sql_statement_facts(sql)
        .returning_stars
        .into_iter()
        .map(|star| (star.relation, star.qualified, star.within_function))
        .collect()
}

#[test]
fn bare_star_records_each_base_relation() {
    assert_eq!(
        stars("SELECT * FROM orders WHERE id = $1"),
        vec![("orders".to_string(), false, None, 1)]
    );
    assert_eq!(
        stars("SELECT * FROM orders o JOIN accounts a ON a.id = o.account_id"),
        vec![
            ("orders".to_string(), false, None, 1),
            ("accounts".to_string(), false, None, 1),
        ]
    );
}

#[test]
fn qualified_star_uses_the_alias_relation() {
    assert_eq!(
        stars("SELECT o.*, a.name FROM orders o JOIN accounts a ON a.id = o.account_id"),
        vec![("orders".to_string(), true, None, 1)]
    );
}

#[test]
fn schema_qualified_name_keeps_the_table() {
    assert_eq!(
        stars("SELECT * FROM public.orders"),
        vec![("orders".to_string(), false, None, 1)]
    );
}

#[test]
fn exists_count_and_whole_row_functions_are_distinguished() {
    assert!(stars("SELECT EXISTS (SELECT * FROM orders WHERE account_id = $1)").is_empty());
    assert!(stars("SELECT NOT EXISTS (SELECT * FROM orders WHERE account_id = $1)").is_empty());
    assert!(stars("SELECT COUNT(*) FROM orders").is_empty());
    assert_eq!(
        stars("SELECT row_to_json(o.*) FROM orders o WHERE id = $1"),
        vec![(
            "orders".to_string(),
            true,
            Some("row_to_json".to_string()),
            1
        )]
    );
    assert_eq!(
        stars("SELECT ROW_TO_JSON(o.*) FROM orders o"),
        vec![(
            "orders".to_string(),
            true,
            Some("row_to_json".to_string()),
            1
        )]
    );
}

#[test]
fn cte_and_derived_stars_are_not_base_relations() {
    assert_eq!(
        stars("WITH recent AS (SELECT * FROM orders WHERE id > $1) SELECT id FROM recent"),
        vec![("orders".to_string(), false, None, 1)]
    );
    assert!(stars("WITH r AS (SELECT id, status FROM orders) SELECT * FROM r").is_empty());
    assert!(stars("WITH recent AS (SELECT id FROM orders) SELECT t.* FROM recent t").is_empty());
    assert_eq!(
        stars("SELECT * FROM (SELECT * FROM orders) s"),
        vec![("orders".to_string(), false, None, 1)]
    );
}

#[test]
fn returning_stars_follow_the_target_table() {
    assert_eq!(
        returning("UPDATE orders SET status = 'paid' WHERE id = $1 RETURNING *"),
        vec![("orders".to_string(), false, None)]
    );
    assert_eq!(
        returning("UPDATE orders o SET status = 'paid' WHERE id = $1 RETURNING o.*"),
        vec![("orders".to_string(), true, None)]
    );
    assert_eq!(
        returning("INSERT INTO orders (id) VALUES (1) RETURNING *"),
        vec![("orders".to_string(), false, None)]
    );
    assert!(returning("UPDATE orders SET status = 'paid' WHERE id = $1 RETURNING id").is_empty());
    assert_eq!(
        returning("UPDATE orders o SET status = 'paid' RETURNING row_to_json(o.*)"),
        vec![("orders".to_string(), true, Some("row_to_json".to_string()))]
    );
}
