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
