use crate::codebase::postgres::{extract_sql_statement_facts, SqlStatementFileFacts};

fn facts(name: &str) -> SqlStatementFileFacts {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join(format!("sql/table-select-{name}.sql")),
    )
    .unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{name}");
    assert_eq!(facts.bounds.len(), 1, "{name}");
    facts
}

#[test]
fn right_table_arms_preserve_exact_source_identity_and_line() {
    for (name, relation) in [("quoted", "\"Topics\""), ("folded", "topics")] {
        let facts = facts(name);
        let arm = facts
            .selects
            .iter()
            .find(|select| select.tables == [relation])
            .expect("TABLE arm Select fact");
        assert_eq!(arm.line, 1);
        assert_eq!(arm.relations.len(), 1);
        assert_eq!(arm.relations[0].table, relation);
        assert_eq!(arm.star_projections.len(), 1);
        assert_eq!(arm.star_projections[0].relation, relation);
        assert!(facts.limit_uses.is_empty());
        assert!(contains_bound_table(&facts.bounds[0].query, relation));

        let sql = std::fs::read_to_string(
            crate::test_support::rule_fixture_root("postgres-bounded-statements")
                .join(format!("sql/table-select-{name}.sql")),
        )
        .unwrap();
        // The prepared request keeps original token positions even when sqlparser's
        // TABLE AST supplies no source span for the separate source API.
        let prepared = crate::codebase::postgres::parse::PreparedSql::new(&sql);
        let table = prepared.tokens().iter().find(|token| {
            matches!(&token.token, sqlparser::tokenizer::Token::Word(word) if word.keyword == sqlparser::keywords::Keyword::TABLE)
        }).expect("source TABLE token");
        assert_eq!(table.span.start.line, 1);
        assert_eq!(
            table.span.start.column as usize,
            sql.find("TABLE").unwrap() + 1
        );
    }
}

#[test]
fn table_arm_without_source_tokens_does_not_guess_relation_identity() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/table-select-quoted.sql"),
    )
    .unwrap();
    let prepared = crate::codebase::postgres::parse::PreparedSql::new(&sql);
    let statements = prepared.parse().unwrap();
    let mut cursor = crate::codebase::postgres::statements::bounds::TableTokenCursor::default();
    let mut out = Vec::new();
    super::collect(&statements[0], Some(&mut cursor), &mut out);
    assert!(out.is_empty());
}

fn contains_bound_table(
    query: &crate::codebase::postgres::statements::SqlBoundQuery,
    relation: &str,
) -> bool {
    use crate::codebase::postgres::statements::SqlBoundItemKind;
    query.items.iter().any(|item| match &item.kind {
        SqlBoundItemKind::Table(name) => name == relation,
        SqlBoundItemKind::Query(query) => contains_bound_table(query, relation),
        _ => false,
    })
}

#[test]
fn left_table_arm_keeps_its_existing_select_fact() {
    let facts = facts("left");
    assert_eq!(
        facts
            .selects
            .iter()
            .filter(|select| select.tables == ["topics"])
            .count(),
        1
    );
}

#[test]
fn table_cte_aliases_do_not_become_physical_relations() {
    for (name, alias) in [("cte", "topics"), ("recursive", "later_cte")] {
        let facts = facts(name);
        assert!(
            facts
                .selects
                .iter()
                .all(|select| { !select.tables.iter().any(|table| table == alias) }),
            "{name}: {:?}",
            facts.selects
        );
    }
}

#[test]
fn table_arm_inside_exists_has_no_explicit_columns_star() {
    let facts = facts("exists");
    let arm = facts
        .selects
        .iter()
        .find(|select| select.tables == ["topics"])
        .expect("TABLE arm Select fact");
    assert!(arm.star_projections.is_empty());
    assert_eq!(arm.relations[0].table, "topics");
}

#[test]
fn recovered_do_body_table_arm_uses_its_source_tokens_even_without_bounds() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-table-quoted-do.sql"),
    )
    .unwrap();
    let prepared = crate::codebase::postgres::parse::PreparedSql::new(&sql);
    assert!(!prepared.tokens().iter().any(|token| {
        matches!(&token.token, sqlparser::tokenizer::Token::Word(word) if word.keyword == sqlparser::keywords::Keyword::TABLE)
    }));
    for collect_bounds in [true, false] {
        let facts = crate::codebase::postgres::statements::extract_sql_statement_facts_with_bounds(
            &sql,
            collect_bounds,
        );
        assert!(facts.parse_failed);
        let arm = facts
            .selects
            .iter()
            .find(|select| select.tables == ["\"Accounts\""])
            .expect("recovered TABLE arm Select fact");
        assert_eq!(arm.line, 5);
        assert_eq!(arm.star_projections[0].relation, "\"Accounts\"");
        assert_eq!(facts.bounds.is_empty(), !collect_bounds);
    }
}
