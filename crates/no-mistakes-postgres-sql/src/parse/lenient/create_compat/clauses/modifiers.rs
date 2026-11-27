use super::{at_column_depth, mask, pg_ident, word};
use sqlparser::{
    dialect::PostgreSqlDialect,
    parser::Parser,
    tokenizer::{Token, TokenWithSpan},
};

pub(crate) fn column_modifiers(tokens: &mut [TokenWithSpan], code: &[usize], body: usize) -> bool {
    for at in body + 1..code.len().saturating_sub(1) {
        let compression = word(&tokens[code[at]].token, "COMPRESSION");
        let storage = word(&tokens[code[at]].token, "STORAGE");
        if !(compression || storage) || !at_column_depth(tokens, code, body, at) {
            continue;
        }
        let valid = if compression {
            pg_ident::method(&tokens[code[at + 1]].token)
        } else {
            ["PLAIN", "EXTERNAL", "EXTENDED", "MAIN"]
                .iter()
                .any(|name| word(&tokens[code[at + 1]].token, name))
        };
        if !valid
            || compression
                && code
                    .get(at + 2)
                    .is_some_and(|i| word(&tokens[*i].token, "STORAGE"))
        {
            continue;
        }
        let start = (body + 1..at)
            .rev()
            .find(|pos| {
                tokens[code[*pos]].token == Token::Comma
                    && at_column_depth(tokens, code, body, *pos)
            })
            .map_or(body + 1, |pos| pos + 1);
        if !pg_ident::col_id(&tokens[code[start]].token) {
            continue;
        }
        let prefix = code[start..at]
            .iter()
            .map(|index| tokens[*index].token.clone())
            .collect();
        let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(prefix);
        // Storage and compression precede constraints/defaults in PostgreSQL.
        if parser.parse_identifier().is_ok()
            && parser.parse_data_type().is_ok()
            && parser.peek_token().token == Token::EOF
        {
            mask(tokens, code, at, at + 2);
            return true;
        }
    }
    false
}
