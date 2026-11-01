use super::super::{facts, query, shape};
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::postgres::statements::{
    SqlBoundItemKind, SqlBoundKind, SqlBoundQuery, SqlPinSource,
};

fn parsed(name: &str) -> String {
    let sql = super::fixture(name);
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{name}: {facts:?}");
    sql
}

fn select_query(sql: &str) -> SqlBoundQuery {
    facts(sql)
        .into_iter()
        .find(|fact| fact.kind == SqlBoundKind::Select)
        .unwrap_or_else(|| panic!("{}", shape(sql).join(" | ")))
        .query
}

fn messages_key_pins(query: &SqlBoundQuery) -> Vec<String> {
    let mut pins = Vec::new();
    collect(query, &mut pins);
    pins
}

fn collect(query: &SqlBoundQuery, pins: &mut Vec<String>) {
    for item in &query.items {
        if let SqlBoundItemKind::Table(name) = &item.kind {
            if name == "messages" {
                for pin in &item.pins {
                    if let SqlPinSource::Items(indexes)
                    | SqlPinSource::Array { items: indexes, .. } = &pin.source
                    {
                        pins.push(format!("{}={indexes:?}", pin.column));
                    }
                }
            }
        }
        if let SqlBoundItemKind::Query(inner) = &item.kind {
            collect(inner, pins);
        }
    }
}

fn contains_capped_query(query: &SqlBoundQuery) -> bool {
    query.capped
        || query.items.iter().any(|item| {
            matches!(&item.kind, SqlBoundItemKind::Query(inner) if inner.capped || contains_capped_query(inner))
        })
}

#[test]
fn a_positional_limit_keeps_page_key_pins_on_the_returning_read() {
    for name in ["issue-2961-page-limit-6.sql", "issue-2961-page-limit-5.sql"] {
        let select = select_query(&parsed(name));
        let pins = messages_key_pins(&select);
        assert!(contains_capped_query(&select), "{name}");
        assert!(!pins.is_empty(), "{name}: {pins:?} {}", query(&select));
    }
}

#[test]
fn a_page_without_a_limit_drops_the_returning_key_pins() {
    let select = select_query(&parsed("issue-2961-page-uncapped.sql"));
    assert!(!contains_capped_query(&select), "{}", query(&select));
    assert!(messages_key_pins(&select).is_empty(), "{}", query(&select));
}

#[test]
fn array_pins_to_a_capped_sibling_survive_returning_projection() {
    let sql = "WITH page AS (SELECT id FROM messages ORDER BY id LIMIT 1), \
        changed AS (UPDATE messages SET deleted_at = now() FROM page \
        WHERE id = ANY(ARRAY[page.id, page.id]) RETURNING id) SELECT * FROM changed";
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed, "{facts:?}");
    let pins = messages_key_pins(&select_query(sql));
    assert!(pins.iter().any(|pin| pin.starts_with("id=")), "{pins:?}");
}

#[test]
fn only_the_capped_sibling_stays_and_its_pin_index_is_remapped() {
    let sql = "WITH extra AS (SELECT id FROM other), \
        page AS (SELECT id FROM messages ORDER BY id LIMIT 1), \
        changed AS (UPDATE messages SET deleted_at = now() FROM extra, page \
        WHERE messages.id = page.id RETURNING id) SELECT * FROM changed";
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed, "{facts:?}");
    let rendered = query(&select_query(sql));
    assert!(rendered.contains("id=#1"), "{rendered}");
    assert!(!rendered.contains("#2"), "{rendered}");
}

#[test]
fn an_empty_returning_projection_is_absent() {
    assert!(super::super::super::returning::project(SqlBoundQuery::default()).is_none());
}
