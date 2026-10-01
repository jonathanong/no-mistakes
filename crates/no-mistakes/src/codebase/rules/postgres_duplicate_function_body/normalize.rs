use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Tokenizer, Word};

pub(super) struct Settings {
    pub(super) normalize_identifiers: bool,
    pub(super) normalize_raise: bool,
    pub(super) keep_identifiers: Vec<String>,
}

pub(super) fn normalized_tokens(body: &str, settings: &Settings) -> Option<Vec<String>> {
    let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, body)
        .tokenize()
        .ok()?
        .into_iter()
        .filter(|token| !matches!(token, Token::EOF | Token::Whitespace(_)))
        .collect::<Vec<_>>();
    if tokens
        .last()
        .is_some_and(|token| matches!(token, Token::SemiColon))
    {
        tokens.pop();
    }
    let mut out = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if settings.normalize_raise && is_word(&tokens[index], "raise") {
            let (level, next) = consume_raise(&tokens, index);
            out.extend(["RAISE".to_string(), level, "?".to_string(), ";".to_string()]);
            index = next;
            continue;
        }
        let call = tokens
            .get(index + 1)
            .is_some_and(|token| matches!(token, Token::LParen));
        out.push(render(&tokens[index], call, settings));
        index += 1;
    }
    Some(out)
}

fn consume_raise(tokens: &[Token], start: usize) -> (String, usize) {
    let mut index = start + 1;
    let level = match tokens.get(index) {
        Some(Token::Word(word)) if is_level(&word.value) => {
            index += 1;
            word.value.to_ascii_uppercase()
        }
        _ => "EXCEPTION".to_string(),
    };
    let mut depth: usize = 0;
    while index < tokens.len() {
        match &tokens[index] {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            Token::SemiColon if depth == 0 => {
                index += 1;
                break;
            }
            _ => {}
        }
        index += 1;
    }
    (level, index)
}

fn is_level(value: &str) -> bool {
    matches!(
        value.to_ascii_uppercase().as_str(),
        "EXCEPTION" | "WARNING" | "NOTICE" | "INFO" | "LOG" | "DEBUG"
    )
}

fn is_word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.value.eq_ignore_ascii_case(expected))
}

fn render(token: &Token, call: bool, settings: &Settings) -> String {
    match token {
        Token::Number(_, _) => "0".to_string(),
        Token::Word(word) => render_word(word, call, settings),
        Token::DoubleQuotedString(value) => render_name(value, call, settings),
        other if is_string(other) => "'?'".to_string(),
        other => other.to_string(),
    }
}

fn render_word(word: &Word, call: bool, settings: &Settings) -> String {
    if word.quote_style.is_none() && word.keyword != Keyword::NoKeyword {
        return word.value.to_ascii_uppercase();
    }
    render_name(&word.value, call, settings)
}

fn render_name(value: &str, call: bool, settings: &Settings) -> String {
    if call {
        return value.to_string();
    }
    let upper = value.to_ascii_uppercase();
    if upper == "NEW" || upper == "OLD" || upper.starts_with("TG_") {
        return upper;
    }
    if settings.keep_identifiers.iter().any(|kept| kept == &upper) {
        return upper;
    }
    if settings.normalize_identifiers {
        "ID".to_string()
    } else {
        value.to_string()
    }
}

fn is_string(token: &Token) -> bool {
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
