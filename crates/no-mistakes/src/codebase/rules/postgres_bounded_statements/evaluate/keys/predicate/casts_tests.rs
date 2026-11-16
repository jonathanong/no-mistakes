use super::super::shape::{atom_with, membership_with, Atom, Lit};
use super::preserves;
use crate::codebase::postgres::parse_postgres_expression;
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{CharLengthUnits, CharacterLength, DataType};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn catalog() -> SchemaCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3394");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

fn atom(expr: &sqlparser::ast::Expr, catalog: &SchemaCatalog) -> Option<Atom> {
    atom_with(expr, catalog, &BTreeMap::new())
}

fn membership(expr: &sqlparser::ast::Expr, catalog: &SchemaCatalog) -> Option<Atom> {
    membership_with(expr, catalog, &BTreeMap::new())
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
    let varchar = BTreeMap::from([("label".to_owned(), "character varying".to_owned())]);
    assert!(atom_with(
        &parsed("(label)::text = ANY (ARRAY['pending'::text, 'ready'::text])"),
        &catalog,
        &varchar
    )
    .is_some());
    let citext = BTreeMap::from([("folded".to_owned(), "citext".to_owned())]);
    assert!(atom_with(&parsed("folded = 'a'::text"), &catalog, &citext).is_none());
    assert!(atom_with(&parsed("(folded)::text = 'a'::text"), &catalog, &citext).is_none());
    let character = BTreeMap::from([("code".to_owned(), "character".to_owned())]);
    assert!(atom_with(&parsed("code = 'a'::text"), &catalog, &character).is_none());
    assert!(atom_with(&parsed("(code)::text = 'a '::text"), &catalog, &character).is_none());
    let qualified = BTreeMap::from([("label".to_owned(), "pg_catalog.text".to_owned())]);
    assert!(atom_with(
        &parsed("(label)::text = 'pending'::text"),
        &catalog,
        &qualified
    )
    .is_some());
    let nvarchar = BTreeMap::from([("label".to_owned(), "nvarchar".to_owned())]);
    assert!(atom_with(
        &parsed("(label)::text = 'pending'::text"),
        &catalog,
        &nvarchar
    )
    .is_none());
    let domain = BTreeMap::from([("folded".to_owned(), "ci_email".to_owned())]);
    assert!(atom_with(&parsed("(folded)::text = 'a'::text"), &catalog, &domain).is_none());
    let qualified_domain = BTreeMap::from([("folded".to_owned(), "public.ci_email".to_owned())]);
    assert!(atom_with(
        &parsed("(folded)::text = 'a'::text"),
        &catalog,
        &qualified_domain
    )
    .is_none());
    assert!(atom_with(&parsed("label = 'pending'::citext"), &catalog, &varchar).is_none());
}
