use super::source_column;
use crate::codebase::postgres::{
    parse_postgres_expression, CatalogColumn, CatalogTable, RelationKind,
};

fn table(columns: &[(&str, &str)]) -> CatalogTable {
    CatalogTable {
        name: "t".to_string(),
        relation_kind: RelationKind::Table,
        partition_of: None,
        comment: None,
        columns: columns
            .iter()
            .enumerate()
            .map(|(index, (name, data_type))| CatalogColumn {
                name: (*name).to_string(),
                data_type: (*data_type).to_string(),
                nullable: true,
                default_expression: None,
                generated: None,
                generated_expression: None,
                identity: None,
                comment: None,
                ordinal_position: index as u32 + 1,
            })
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

#[test]
fn column_equality_fixes_only_allowlisted_shapes() {
    let columns = table(&[
        ("username", "text"),
        ("email", "character varying(64)"),
        ("n", "integer"),
        ("small", "smallint"),
        ("flag", "boolean"),
        ("token", "uuid"),
        ("code", "text"),
        ("code_char", "char"),
        ("span", "int4range"),
        ("spans", "int4multirange"),
        ("blob", "bytea"),
        ("custom", "public.text"),
    ]);
    let cases = [
        ("lower(username)", Some("username")),
        ("UPPER(username)", Some("username")),
        ("btrim(username)", Some("username")),
        ("pg_catalog.lower(username)", Some("username")),
        ("pg_catalog.btrim(username)", Some("username")),
        ("lower((username))", Some("username")),
        ("\"lower\"(username)", None),
        ("app.lower(username)", None),
        ("btrim(username, 'x')", None),
        ("lower(username || '')", None),
        ("length(username)", None),
        ("lower(absent)", None),
        ("username", None),
        ("lower(email)", Some("email")),
        ("email::pg_catalog.text", Some("email")),
        ("n::bigint", Some("n")),
        ("CAST(n AS bigint)", Some("n")),
        ("small::integer", Some("small")),
        ("flag::boolean", Some("flag")),
        ("token::uuid", Some("token")),
        ("code::char", None),
        ("token::text", None),
        ("lower(username)::text", None),
        ("(token::text)", None),
        ("custom::text", None),
        ("lower(username) OVER ()", None),
        ("lower(username) FILTER (WHERE true)", None),
        ("lower(username) IGNORE NULLS", None),
        ("lower(username) WITHIN GROUP (ORDER BY username)", None),
        ("lower(DISTINCT username)", None),
        ("lower(username ORDER BY username)", None),
        ("lower(s => username)", None),
        ("lower(*)", None),
        ("TRY_CAST(username AS text)", None),
        ("SAFE_CAST(username AS text)", None),
        ("CAST(username AS text FORMAT 'x')", None),
        ("a.b.lower(username)", None),
        ("\"pg_catalog\".lower(username)", None),
        ("pg_catalog.\"lower\"(username)", None),
        ("lower(code_char)", None),
        ("lower(span)", None),
        ("upper(span)", None),
        ("lower(spans)", None),
        ("btrim(blob)", None),
    ];
    for (expression, expected) in cases {
        let parsed =
            parse_postgres_expression(expression).unwrap_or_else(|| panic!("parse {expression}"));
        assert_eq!(
            source_column(&parsed, &columns).as_deref(),
            expected,
            "{expression}"
        );
    }
}
