use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-parse-failure-locations/fixture")
            .join(name),
    )
    .unwrap()
}

#[test]
fn canonical_capture_ignores_chunks_without_code_or_source_tokens() {
    for name in ["canonical-empty.sql", "canonical-comments.sql"] {
        let source = fixture(name);
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
            .tokenize_with_location()
            .unwrap();
        let code: Vec<_> = tokens
            .into_iter()
            .filter(|token| token.token != sqlparser::tokenizer::Token::SemiColon)
            .collect();
        assert!(super::unrecoverable_chunk(&code, 0).is_none());
    }
    let source = fixture("failed-insert.sql");
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
        .tokenize()
        .unwrap();
    // An unlocated parser fragment cannot invent a source-backed failure occurrence.
    let parsed = super::super::parse_chunks_with_function_calls(
        vec![tokens],
        &[],
        true,
        &crate::codebase::postgres::parse::RecursiveViews::default(),
    );
    assert!(crate::codebase::postgres::migration::recovery_failures(&parsed).is_empty());
}
