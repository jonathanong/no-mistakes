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
        &[],
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
    assert_eq!(
        header::child_start(&tokens[..1], PostgresSqlWrapperKind::Prepare),
        1
    );
    let missing = project(&ast, &tokens[..1], &context, 0, None).unwrap();
    assert!(!missing.complete);
    assert!(!missing.diagnostics.is_empty());
}
