use sqlparser::tokenizer::Token;

pub(super) fn raise_statement(tokens: &[Token], index: usize) -> bool {
    let Some(previous) = index.checked_sub(1).and_then(|prior| tokens.get(prior)) else {
        return true;
    };
    match previous {
        Token::SemiColon => true,
        Token::Word(word) => matches!(
            word.value.to_ascii_uppercase().as_str(),
            "BEGIN" | "THEN" | "ELSE" | "LOOP" | "ATOMIC"
        ),
        _ => false,
    }
}

pub(super) fn inside_operator(tokens: &[Token], index: usize) -> bool {
    let mut depth = 0i32;
    let mut saw_operator = false;
    for token in tokens.iter().take(index) {
        match token {
            Token::Word(word) if depth == 0 && word.value.eq_ignore_ascii_case("operator") => {
                saw_operator = true;
            }
            Token::LParen if saw_operator && depth == 0 => {
                depth = 1;
                saw_operator = false;
            }
            Token::LParen if depth > 0 => depth += 1,
            Token::RParen if depth > 0 => depth -= 1,
            _ if depth == 0 => saw_operator = false,
            _ => {}
        }
    }
    depth > 0
}

pub(super) fn is_string(token: &Token) -> bool {
    matches!(
        token,
        Token::SingleQuotedString(_)
            | Token::DollarQuotedString(_)
            | Token::EscapedStringLiteral(_)
            | Token::NationalStringLiteral(_)
            | Token::UnicodeStringLiteral(_)
            | Token::HexStringLiteral(_)
            | Token::SingleQuotedByteStringLiteral(_)
            | Token::DoubleQuotedByteStringLiteral(_)
            | Token::SingleQuotedRawStringLiteral(_)
            | Token::DoubleQuotedRawStringLiteral(_)
            | Token::TripleSingleQuotedString(_)
            | Token::TripleDoubleQuotedString(_)
            | Token::TripleSingleQuotedByteStringLiteral(_)
            | Token::TripleDoubleQuotedByteStringLiteral(_)
            | Token::TripleSingleQuotedRawStringLiteral(_)
            | Token::TripleDoubleQuotedRawStringLiteral(_)
            | Token::QuoteDelimitedStringLiteral(_)
            | Token::NationalQuoteDelimitedStringLiteral(_)
    )
}
