use crate::codebase::postgres::statements::extract_sql_statement_facts;

#[test]
fn nested_exists_set_operations_are_reported_once() {
    // The nested EXISTS is reachable from the assignment walk and the SELECT walk.
    let sql = "INSERT INTO t (id, ok) VALUES (1, true) ON CONFLICT (id) DO UPDATE SET ok = (SELECT EXISTS (SELECT 1 FROM a WHERE a.id = t.id UNION SELECT 1 FROM b WHERE b.id = t.id));";
    let facts = extract_sql_statement_facts(sql);
    let count: usize = facts
        .selects
        .iter()
        .map(|select| select.exists_set_operations.len())
        .sum();
    assert_eq!(count, 1);
}

#[test]
fn dollar_identifiers_do_not_hide_the_exists_keyword() {
    let sql = "SELECT 'exists', foo$tag$ FROM t WHERE EXISTS (SELECT 1 FROM a WHERE a.id = t.id UNION SELECT 1 FROM b WHERE b.id = t.id);";
    let facts = extract_sql_statement_facts(sql);
    let exists: Vec<_> = facts
        .selects
        .iter()
        .flat_map(|select| &select.exists_set_operations)
        .collect();
    assert_eq!(exists.len(), 1);
    assert_eq!(exists[0].column, 40);
}
