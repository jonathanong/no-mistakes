use super::*;
use sqlparser::dialect::PostgreSqlDialect;

#[test]
fn missing_child_tokens_and_projection_depth_fail_closed() {
    let sql = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/wrappers.sql"),
    )
    .unwrap();
    let source = PostgresSqlSource {
        sql,
        file_name: None,
    };
    let locations = Locations::new(&source.sql);
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&source.sql);
    let context = Context::new(
        &source,
        &locations,
        &prepared.recursive_views,
        &[],
        &[],
        child::Markers {
            inserts: &[],
            index_only: Vec::new(),
        },
        super::super::metadata_preparation::Comments::new(),
    );
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let ast = parser.parse_statement().unwrap();
    let tokens = (0..parser.index())
        .map(|index| parser.token_at(index).clone())
        .collect::<Vec<_>>();
    let bounded = project(&ast, &tokens, &context, 16, None).unwrap();
    assert!(!bounded.complete);
    assert!(bounded.statements.is_empty());
    assert!(bounded.diagnostics[0].message.contains("safety limit"));
    assert!(span(&[], &context).is_none());
    assert!(context.statement(&ast, &[], 0, 0, None).is_err());
    // Reversed malformed bounds retain the old empty-range fallback.
    assert_eq!(
        context.source_end(tokens[1].span.end, tokens[0].span.start),
        tokens[0].span.start
    );
    assert_eq!(
        header::child_start(&tokens[..1], PostgresSqlWrapperKind::Prepare),
        1
    );
    let missing = project(&ast, &tokens[..1], &context, 0, None).unwrap();
    assert!(!missing.complete);
    assert!(!missing.diagnostics.is_empty());
}

#[test]
fn prepared_legacy_aliases_do_not_rewrite_expression_and_returning_identifiers() {
    let sql = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/wrapper-noncommand.sql"),
    )
    .unwrap();
    let mut prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    prepare(&mut prepared.tokens);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let statements = parser.parse_statements().unwrap();
    for statement in &statements[..2] {
        let Statement::Query(query) = statement else {
            panic!("query expected")
        };
        let sqlparser::ast::SetExpr::Select(select) = query.body.as_ref() else {
            panic!("SELECT expected")
        };
        let sqlparser::ast::SelectItem::ExprWithAlias { alias, .. } = &select.projection[0] else {
            panic!("alias expected")
        };
        assert_eq!(alias.value, "analyse");
    }
    let Statement::Insert(insert) = &statements[2] else {
        panic!("INSERT expected")
    };
    let sqlparser::ast::SelectItem::ExprWithAlias { alias, .. } =
        &insert.returning.as_ref().unwrap()[0]
    else {
        panic!("alias expected")
    };
    assert_eq!(alias.value, "analyse");
}
