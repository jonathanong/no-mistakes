use super::rewrite;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, Tokenizer};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
            .join("query")
            .join(name),
    )
    .unwrap()
}

#[test]
fn rewrites_array_targets_but_keeps_conflict_clause() {
    let sql = fixture("array-target-valid.sql");
    for statement in sql.split(';').map(str::trim).filter(|sql| !sql.is_empty()) {
        let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, statement)
            .tokenize_with_location()
            .unwrap();
        rewrite(&mut tokens);
        let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
        let ast = parser.parse_statement().unwrap();
        assert_eq!(parser.peek_token().token, Token::EOF);
        if let Statement::Insert(insert) = ast {
            assert_eq!(
                insert.on.is_some(),
                statement.to_ascii_lowercase().contains("on conflict"),
                "{statement}"
            );
        } else {
            assert!(matches!(ast, Statement::Update { .. }), "{statement}");
        }
    }
}

#[test]
fn malformed_array_targets_do_not_become_statements() {
    let sql = fixture("array-target-malformed.sql");
    for statement in sql.split(';').map(str::trim).filter(|sql| !sql.is_empty()) {
        let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, statement)
            .tokenize_with_location()
            .unwrap();
        rewrite(&mut tokens);
        let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
        assert!(
            parser.parse_statement().is_err() || parser.peek_token().token != Token::EOF,
            "{statement}"
        );
    }
}

#[test]
fn array_reads_in_insert_sources_and_assignment_rhs_keep_their_ast() {
    let sql = fixture("array-read-preserved.sql");
    for (line, statement) in sql.lines().enumerate() {
        let statement = statement.trim_end_matches(';');
        let original = Parser::parse_sql(&PostgreSqlDialect {}, statement);
        let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, statement)
            .tokenize_with_location()
            .unwrap();
        rewrite(&mut tokens);
        let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
        let ast = parser.parse_statement().unwrap();
        assert_eq!(parser.peek_token().token, Token::EOF, "{statement}");
        if line < 3 {
            assert_eq!(original.unwrap(), vec![ast], "{statement}");
        } else {
            // The target selector is hidden for sqlparser, but the RHS read stays.
            assert!(ast.to_string().contains("arr[1]"), "{ast}");
            assert!(!ast.to_string().contains("f1[2]"), "{ast}");
        }
    }
}
