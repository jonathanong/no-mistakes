//! Project literal procedural SQL without evaluating an expression or executing SQL.
use super::{locations::Locations, types::*};
use sqlparser::{parser::Parser, tokenizer::Token};

pub(super) fn collect(
    parser: &mut Parser<'_>,
    source: &PostgresSqlSource,
    locations: &Locations<'_>,
    depth: usize,
) -> Result<PostgresSqlStatementKind, String> {
    parser.next_token();
    let literal = parser.next_token();
    let (decoded_sql, body_encoding) = match literal.token {
        Token::DollarQuotedString(value) => (value.value, PostgresSqlBodyEncoding::DollarQuoted),
        Token::EscapedStringLiteral(value) => (value, PostgresSqlBodyEncoding::EscapedString),
        Token::SingleQuotedString(value) => (value, PostgresSqlBodyEncoding::SingleQuoted),
        _ => {
            while !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
                parser.next_token();
            }
            return Ok(PostgresSqlStatementKind::Other);
        }
    };
    // Concatenation, USING, INTO, and format calls are deliberately unsupported.
    if !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
        while !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
            parser.next_token();
        }
        return Ok(PostgresSqlStatementKind::Other);
    }
    let literal_span = locations
        .span(literal.span)
        .ok_or("EXECUTE literal source span is unavailable")?;
    let mut execute = PostgresSqlLiteralExecute {
        literal_span,
        body_encoding,
        decoded_sql,
        statements: Vec::new(),
        diagnostics: Vec::new(),
        complete: false,
    };
    if depth >= 64 {
        execute.diagnostics.push(PostgresSqlDiagnostic {
            message: "Literal EXECUTE nesting exceeds the parser safety limit".into(),
            span: None,
        });
    } else {
        let nested_source = PostgresSqlSource {
            sql: execute.decoded_sql.clone(),
            file_name: source.file_name.clone(),
        };
        let prepared =
            crate::codebase::postgres::parse::prepare_postgres_tokens(&nested_source.sql);
        let nested = super::parsing::collect_program(
            &nested_source,
            prepared,
            &Locations::new(&nested_source.sql),
            depth + 1,
            false,
        );
        execute.statements = nested.statements;
        execute.diagnostics = nested.diagnostics;
        execute.complete = execute.diagnostics.is_empty()
            && execute
                .statements
                .iter()
                .all(|value| super::completeness::statement(&value.facts));
    }
    Ok(PostgresSqlStatementKind::LiteralExecute { execute })
}
