//! Project literal procedural SQL without evaluating an expression or executing SQL.
use super::{locations::Locations, types::*};
use sqlparser::{
    ast::{BinaryOperator, Expr, Value},
    keywords::Keyword,
    parser::Parser,
    tokenizer::{Span, Token},
};

pub(super) fn collect(
    parser: &mut Parser<'_>,
    source: &PostgresSqlSource,
    locations: &Locations<'_>,
    depth: usize,
) -> Result<PostgresSqlStatementKind, String> {
    parser.next_token();
    let start = parser.peek_token().span.start;
    let command = match parser.try_parse(|parser| parser.parse_expr()) {
        Ok(command) => command,
        Err(_) => return unsupported(parser),
    };
    let command_span = Span {
        start,
        end: parser.get_current_token().span.end,
    };
    let Some((decoded_sql, body_encoding)) = literal_command(&command, 0) else {
        return unsupported(parser);
    };
    let using = if parser.parse_keyword(Keyword::USING) {
        let values =
            match parser.try_parse(|parser| parser.parse_comma_separated(Parser::parse_expr)) {
                Ok(values) => values,
                Err(_) => return unsupported(parser),
            };
        values
            .iter()
            .map(|expr| super::expressions::expression(expr, locations))
            .collect()
    } else {
        Vec::new()
    };
    // INTO and all remaining modifiers still require unsupported procedural semantics.
    if !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
        return unsupported(parser);
    }
    let literal_span = locations
        .span(command_span)
        .ok_or("EXECUTE literal source span is unavailable")?;
    let mut execute = PostgresSqlLiteralExecute {
        literal_span,
        body_encoding,
        decoded_sql,
        using,
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
        execute.complete = execute
            .using
            .iter()
            .all(|expression| expression.children_complete)
            && execute.diagnostics.is_empty()
            && execute
                .statements
                .iter()
                .all(|value| super::completeness::statement(&value.facts));
    }
    Ok(PostgresSqlStatementKind::LiteralExecute { execute })
}

#[cfg(test)]
mod tests;

fn unsupported(parser: &mut Parser<'_>) -> Result<PostgresSqlStatementKind, String> {
    while !matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
        parser.next_token();
    }
    Ok(PostgresSqlStatementKind::Other)
}

fn literal_command(expr: &Expr, depth: u8) -> Option<(String, PostgresSqlExecuteEncoding)> {
    if depth >= 64 {
        return None;
    }
    match expr {
        Expr::Value(value) => match &value.value {
            Value::DollarQuotedString(value) => Some((
                value.value.clone(),
                PostgresSqlExecuteEncoding::DollarQuoted,
            )),
            Value::EscapedStringLiteral(value) => {
                Some((value.clone(), PostgresSqlExecuteEncoding::EscapedString))
            }
            Value::SingleQuotedString(value) => {
                Some((value.clone(), PostgresSqlExecuteEncoding::SingleQuoted))
            }
            _ => None,
        },
        Expr::Nested(inner) => literal_command(inner, depth + 1),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => {
            let (mut left, _) = literal_command(left, depth + 1)?;
            let (right, _) = literal_command(right, depth + 1)?;
            left.push_str(&right);
            Some((left, PostgresSqlExecuteEncoding::Concatenated))
        }
        _ => None,
    }
}
