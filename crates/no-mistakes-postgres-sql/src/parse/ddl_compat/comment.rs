//! PostgreSQL COMMENT accepts dollar strings and newline-joined SQL literals.
use super::word;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn rewrite(code: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    if !code.first().is_some_and(|token| word(token, "COMMENT")) {
        return None;
    }
    let at = code.iter().rposition(|token| word(token, "IS"))? + 1;
    let literals = &code[at..];
    let first = literals.first()?;
    let value = match &first.token {
        Token::DollarQuotedString(value) if literals.len() == 1 => value.value.clone(),
        Token::SingleQuotedString(value) if literals.len() > 1 => {
            let mut value = value.clone();
            for pair in literals.windows(2) {
                // A space alone is not PostgreSQL's implicit literal concatenation.
                if pair[0].span.end.line >= pair[1].span.start.line {
                    return None;
                }
                let Token::SingleQuotedString(next) = &pair[1].token else {
                    return None;
                };
                value.push_str(next);
            }
            value
        }
        _ => return None,
    };
    let mut rewritten = code[..at].to_vec();
    rewritten.push(TokenWithSpan::new(
        Token::SingleQuotedString(value),
        first.span,
    ));
    let rewritten = super::rewrite(&rewritten).unwrap_or(rewritten);
    let mut parser =
        Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(rewritten.clone());
    let valid = parser.parse_statement().is_ok() && parser.peek_token().token == Token::EOF;
    if valid {
        Some(rewritten)
    } else if super::metadata::validate(&rewritten) {
        // Trigger comments have no native AST; the normal metadata path owns them.
        Some(
            ["DISCARD", "PLANS"]
                .map(|value| TokenWithSpan::new(Token::make_keyword(value), code[0].span))
                .to_vec(),
        )
    } else {
        None
    }
}
