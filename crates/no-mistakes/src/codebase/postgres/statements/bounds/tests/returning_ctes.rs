use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundKind, SqlPinSource};

fn fixture(name: &str) -> String {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("sql/cte-returning")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn returning_cte_projects_only_the_modified_target_row_bound() {
    for (name, kind) in [
        ("update-pk.sql", SqlBoundKind::Update),
        ("delete-pk.sql", SqlBoundKind::Delete),
    ] {
        let found = super::super::facts(&fixture(name));
        assert_eq!(found.len(), 2, "{name}");
        assert_eq!(found[0].kind, kind);
        assert_eq!(found[1].kind, SqlBoundKind::Update);
        let source = &found[1].query.items[1];
        let SqlBoundItemKind::Query(bound) = &source.kind else {
            panic!("{name}: CTE must project a query bound");
        };
        assert_eq!(
            bound.items.len(),
            1,
            "{name}: only the target modifies rows"
        );
        assert!(matches!(&bound.items[0].kind, SqlBoundItemKind::Table(_)));
        assert!(bound.items[0]
            .pins
            .iter()
            .any(|pin| pin.column == "id" && matches!(&pin.source, SqlPinSource::Value)));
    }
}

#[test]
fn unbounded_and_recursive_returning_ctes_keep_the_outer_target_unproved() {
    for name in [
        "unbounded-update.sql",
        "auxiliary-unbounded.sql",
        "recursive-opaque.sql",
    ] {
        let found = super::super::facts(&fixture(name));
        assert_eq!(found.len(), 2, "{name}");
        let source = &found[1].query.items[1];
        if name == "auxiliary-unbounded.sql" {
            let SqlBoundItemKind::Query(bound) = &source.kind else {
                panic!("{name}: CTE must project a query bound");
            };
            // The target-to-auxiliary pin indexed a removed FROM item and must not survive.
            assert!(bound.items[0].pins.is_empty());
        }
        if name == "recursive-opaque.sql" {
            assert!(matches!(&source.kind, SqlBoundItemKind::Query(_)));
            let SqlBoundItemKind::Query(bound) = &source.kind else {
                unreachable!()
            };
            assert!(matches!(&bound.items[0].kind, SqlBoundItemKind::Opaque));
        }
    }
}

#[test]
fn nested_with_names_shadow_catalog_tables_in_returning_projections() {
    for name in ["nested-with-bounded.sql", "nested-with-namesake.sql"] {
        let found = super::super::facts(&fixture(name));
        assert_eq!(found.len(), 2, "{name}");
        let SqlBoundItemKind::Query(cte) = &found[1].query.items[1].kind else {
            panic!("{name}: outer query must read the modifying CTE");
        };
        let pin = cte.items[0]
            .pins
            .iter()
            .find(|pin| pin.column == "id")
            .expect("target key pin");
        let SqlPinSource::Query(pin_query) = &pin.source else {
            panic!("{name}: target pin must retain its inner query");
        };
        // The inner name is a lexical CTE, even when the catalog has a table by that name.
        assert!(matches!(
            &pin_query.items[0].kind,
            SqlBoundItemKind::Query(_)
        ));
    }
}

#[test]
fn unused_modifying_cte_without_returning_stays_opaque() {
    use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};
    let mut statements =
        Parser::parse_sql(&PostgreSqlDialect {}, &fixture("no-returning-unused.sql")).unwrap();
    let Statement::Query(query) = statements.remove(0) else {
        panic!("expected WITH query");
    };
    let outer = super::super::super::Scope::default();
    let scope = super::super::super::query::with_scope(&query, &outer, None);
    let changed = scope.get("changed").expect("CTE in scope");
    assert!(matches!(&changed.items[0].kind, SqlBoundItemKind::Opaque));
}

#[test]
fn repeated_returning_ctes_keep_the_shared_expansion_cap() {
    use crate::codebase::postgres::statements::SqlBoundQuery;
    fn contains_opaque(query: &SqlBoundQuery) -> bool {
        query.items.iter().any(|item| {
            matches!(item.kind, SqlBoundItemKind::Opaque)
                || matches!(&item.kind, SqlBoundItemKind::Query(inner) if contains_opaque(inner))
                || item.pins.iter().any(|pin| {
                    matches!(&pin.source, SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner)
                        if contains_opaque(inner))
                })
        })
    }
    let found = super::super::facts(&fixture("expansion-chain.sql"));
    assert_eq!(found.len(), 14);
    // Without compaction this two-reference chain expands exponentially, with no opaque marker.
    assert!(contains_opaque(&found.last().unwrap().query));
}
