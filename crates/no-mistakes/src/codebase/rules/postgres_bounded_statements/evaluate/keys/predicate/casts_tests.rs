use super::super::shape::{atom, membership, Atom, Lit};
use super::preserves;
use crate::codebase::postgres::parse_postgres_expression;
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{CharLengthUnits, CharacterLength, DataType};
use std::path::PathBuf;

fn catalog() -> SchemaCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3394");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

fn parsed(sql: &str) -> sqlparser::ast::Expr {
    parse_postgres_expression(sql).unwrap_or_else(|| panic!("parse {sql}"))
}

#[test]
fn value_preserving_casts_keep_the_literal() {
    let catalog = catalog();
    assert_eq!(
        atom(&parsed("status = 'pending'::text"), &catalog),
        Some(Atom::Eq(
            "status".to_owned(),
            Lit::Str("pending".to_owned())
        ))
    );
    assert_eq!(
        atom(&parsed("flag = true::boolean"), &catalog),
        Some(Atom::Eq("flag".to_owned(), Lit::Bool(true)))
    );
    assert_eq!(
        atom(&parsed("kind = 'direct'::source_kinds"), &catalog),
        Some(Atom::Eq("kind".to_owned(), Lit::Str("direct".to_owned())))
    );
    assert!(atom(&parsed("kind = 'nope'::source_kinds"), &catalog).is_none());
    assert!(atom(&parsed("kind = 'direct'::not_an_enum"), &catalog).is_none());
    assert!(atom(&parsed("status = 'pending'::varchar(3)"), &catalog).is_none());
    assert!(atom(&parsed("status = 'pending'::char(7)"), &catalog).is_none());
    assert!(atom(&parsed("status = true::varchar"), &catalog).is_none());
    assert!(atom(&parsed("kind = true::source_kinds"), &catalog).is_none());
    assert!(atom(&parsed("status = ('pending'::text[])::text"), &catalog).is_none());
    assert!(atom(
        &parsed("status = ('{\"pending\"}'::text[])::text"),
        &catalog
    )
    .is_none());
    assert_eq!(
        atom(&parsed("kind = 'direct'::\"SourceKinds\""), &catalog),
        Some(Atom::Eq("kind".to_owned(), Lit::Str("direct".to_owned())))
    );
    assert!(atom(&parsed("kind = 'nope'::\"SourceKinds\""), &catalog).is_none());
    assert!(membership(
        &parsed("status = ANY (ARRAY['pending', 'ready']::text[])"),
        &catalog
    )
    .is_some());
    assert!(membership(
        &parsed("status = ANY (ARRAY['pending', 'ready']::varchar(3)[])"),
        &catalog
    )
    .is_none());
    assert!(membership(
        &parsed("status = ANY (ARRAY['pending', 'ready']::text[][])"),
        &catalog
    )
    .is_none());
    assert!(membership(&parsed("status = ANY (ARRAY['pending']::text)"), &catalog).is_none());
    assert!(preserves(
        &DataType::Varchar(Some(CharacterLength::Max)),
        &Lit::Str("ready".to_owned()),
        &catalog
    ));
    assert!(!preserves(
        &DataType::Varchar(Some(CharacterLength::IntegerLength {
            length: 3,
            unit: Some(CharLengthUnits::Octets),
        })),
        &Lit::Str("ab".to_owned()),
        &catalog
    ));
    assert!(!preserves(
        &DataType::Boolean,
        &Lit::Str("true".to_owned()),
        &catalog
    ));
}
