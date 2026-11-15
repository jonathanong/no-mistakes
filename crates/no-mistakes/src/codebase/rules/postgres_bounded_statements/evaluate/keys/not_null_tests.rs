use super::{proves, proves_arm};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use crate::codebase::postgres::{CatalogColumn, CatalogTable, RelationKind};

fn column(name: &str, data_type: &str) -> CatalogColumn {
    CatalogColumn {
        name: name.to_string(),
        data_type: data_type.to_string(),
        nullable: true,
        default_expression: None,
        generated: None,
        generated_expression: None,
        identity: None,
        comment: None,
        ordinal_position: 1,
    }
}

fn table(columns: &[(&str, &str)]) -> CatalogTable {
    CatalogTable {
        name: "t".to_string(),
        relation_kind: RelationKind::Table,
        partition_of: None,
        comment: None,
        columns: columns
            .iter()
            .map(|(name, data_type)| column(name, data_type))
            .collect(),
        primary_key: None,
        foreign_keys: Vec::new(),
        check_constraints: Vec::new(),
        unique_constraints: Vec::new(),
        indexes: Vec::new(),
        triggers: Vec::new(),
        partition_key: None,
    }
}

fn item(raw: &[&str], strict: &[&str]) -> SqlBoundItem {
    let mut item = SqlBoundItem::new(SqlBoundItemKind::Table("t".to_string()), None, (1, 1));
    item.nonnull_columns = raw.iter().map(|name| (*name).to_string()).collect();
    item.strict_nonnull_columns = strict.iter().map(|name| (*name).to_string()).collect();
    item
}

#[test]
fn raw_equality_proves_not_null_for_every_catalog_type() {
    let columns = table(&[("n", "integer"), ("payload", "mytype")]);
    assert!(proves("n", &item(&["n"], &[]), &columns));
    assert!(proves("payload", &item(&["payload"], &[]), &columns));
    assert!(!proves("n", &item(&[], &[]), &columns));
}

#[test]
fn strict_function_equality_proves_not_null_only_for_the_text_family() {
    let columns = table(&[
        ("username", "text"),
        ("name", "character varying(64)"),
        ("plain", "varchar"),
        ("qualified", "pg_catalog.text"),
        ("payload", "mytype"),
        ("n", "integer"),
        ("code", "char"),
        ("custom", "public.text"),
    ]);
    let strict = item(
        &[],
        &[
            "username",
            "name",
            "plain",
            "qualified",
            "payload",
            "n",
            "code",
            "custom",
            "missing",
        ],
    );
    assert!(proves("username", &strict, &columns));
    assert!(proves("name", &strict, &columns));
    assert!(proves("plain", &strict, &columns));
    assert!(proves("qualified", &strict, &columns));
    assert!(!proves("payload", &strict, &columns));
    assert!(!proves("n", &strict, &columns));
    assert!(!proves("code", &strict, &columns));
    assert!(!proves("custom", &strict, &columns));
    assert!(!proves("missing", &strict, &columns));
}

#[test]
fn arm_equality_proves_not_null_for_every_catalog_type() {
    let columns = table(&[("code", "char"), ("folded", "citext"), ("n", "integer")]);
    let plain = item(&[], &[]);
    assert!(proves_arm("code", &plain, &columns, true));
    assert!(proves_arm("folded", &plain, &columns, true));
    assert!(proves_arm("n", &plain, &columns, true));
    assert!(!proves_arm("code", &plain, &columns, false));
}
