use sqlparser::{
    dialect::PostgreSqlDialect,
    tokenizer::{Token, Tokenizer},
};

fn cases(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source/relation-recovery")
        .join(format!("{name}.sql"));
    std::fs::read_to_string(path).unwrap()
}

fn tokens(sql: &str) -> Vec<Token> {
    Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize()
        .unwrap()
        .into_iter()
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect()
}

#[test]
fn complete_relation_compatibility_forms_produce_parseable_queries() {
    for (name, rewrite) in [
        ("xml-valid", super::xml_table as fn(&mut Vec<Token>) -> bool),
        ("sample-valid", |tokens: &mut Vec<Token>| {
            super::sample_null(tokens)
        }),
        ("search-valid", super::cte_search),
    ] {
        for sql in cases(name).lines() {
            let mut tokens = tokens(sql);
            assert!(rewrite(&mut tokens), "{sql}");
            let statements = sqlparser::parser::Parser::new(&PostgreSqlDialect {})
                .with_tokens(tokens)
                .parse_statements()
                .unwrap_or_else(|error| panic!("{sql}: {error}"));
            assert!(
                matches!(statements.as_slice(), [sqlparser::ast::Statement::Query(_)]),
                "{sql}"
            );
        }
    }
}

#[test]
fn incomplete_relation_forms_preserve_the_original_tokens() {
    for (name, rewrite) in [
        (
            "xml-invalid",
            super::xml_table as fn(&mut Vec<Token>) -> bool,
        ),
        ("sample-invalid", |tokens: &mut Vec<Token>| {
            super::sample_null(tokens)
        }),
        ("search-invalid", super::cte_search),
    ] {
        for sql in cases(name).lines() {
            let original = tokens(sql);
            let mut rewritten = original.clone();
            assert!(!rewrite(&mut rewritten), "{sql}");
            assert_eq!(rewritten, original, "{sql}");
        }
    }
}
