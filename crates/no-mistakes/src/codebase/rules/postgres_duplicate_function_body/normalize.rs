use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Tokenizer, Word};
use std::collections::BTreeMap;

pub(super) struct Settings {
    pub(super) normalize_identifiers: bool,
    pub(super) normalize_raise: bool,
    pub(super) keep_identifiers: Vec<String>,
}

pub(super) fn normalized_tokens(
    body: &str,
    settings: &Settings,
    language: Option<&str>,
) -> Option<Vec<String>> {
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
    let plpgsql = language.is_some_and(|name| name.eq_ignore_ascii_case("plpgsql"));
    let mut names = Names::default();
    let mut out = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if plpgsql
            && settings.normalize_raise
            && is_word(&tokens[index], "raise")
            && super::token_class::raise_statement(&tokens, index)
        {
            if bare_raise(&tokens, index) {
                out.extend(["RAISE".to_string(), ";".to_string()]);
                index += 2;
                continue;
            }
            let (level, next) = consume_raise(&tokens, index);
            out.extend(["RAISE".to_string(), level, "?".to_string(), ";".to_string()]);
            index = next;
            continue;
        }
        let call = call_name(&tokens, index)
            || named_argument(&tokens, index)
            || super::token_class::inside_operator(&tokens, index);
        out.push(render(&tokens[index], call, settings, &mut names));
        index += 1;
    }
    Some(out)
}

fn bare_raise(tokens: &[Token], start: usize) -> bool {
    matches!(tokens.get(start + 1), Some(Token::SemiColon) | None)
}

fn call_name(tokens: &[Token], index: usize) -> bool {
    let mut cursor = index;
    loop {
        match tokens.get(cursor + 1) {
            Some(Token::LParen) => return is_name(&tokens[cursor]),
            Some(Token::Period) if tokens.get(cursor + 2).is_some_and(is_name) => cursor += 2,
            _ => return false,
        }
    }
}

fn is_name(token: &Token) -> bool {
    matches!(token, Token::Word(_) | Token::DoubleQuotedString(_))
}

fn named_argument(tokens: &[Token], index: usize) -> bool {
    if !is_name(&tokens[index]) {
        return false;
    }
    match tokens.get(index + 1) {
        Some(Token::RArrow) => true,
        Some(Token::Assignment) => {
            tokens
                .iter()
                .take(index)
                .fold(0i32, |depth, token| match token {
                    Token::LParen => depth + 1,
                    Token::RParen => depth - 1,
                    _ => depth,
                })
                > 0
        }
        _ => false,
    }
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
#[derive(Default)]
struct Names {
    next: u32,
    assigned: BTreeMap<String, String>,
}

fn render(token: &Token, call: bool, settings: &Settings, names: &mut Names) -> String {
    match token {
        Token::Number(_, _) => "0".to_string(),
        Token::Word(word) => render_word(word, call, settings, names),
        Token::DoubleQuotedString(value) => render_name(value, call, true, settings, names),
        other if super::token_class::is_string(other) => "'?'".to_string(),
        other => other.to_string(),
    }
}

fn render_word(word: &Word, call: bool, settings: &Settings, names: &mut Names) -> String {
    if word.quote_style.is_none() && word.keyword != Keyword::NoKeyword && !call {
        return word.value.to_ascii_uppercase();
    }
    render_name(
        &word.value,
        call,
        word.quote_style.is_some(),
        settings,
        names,
    )
}

fn render_name(
    value: &str,
    call: bool,
    quoted: bool,
    settings: &Settings,
    names: &mut Names,
) -> String {
    if call {
        return if quoted {
            value.to_string()
        } else {
            value.to_ascii_lowercase()
        };
    }
    let upper = value.to_ascii_uppercase();
    if upper == "NEW" || upper == "OLD" || upper.starts_with("TG_") {
        return upper;
    }
    if settings.keep_identifiers.iter().any(|kept| kept == &upper) {
        return if quoted {
            format!("\"{value}\"")
        } else {
            upper
        };
    }
    if settings.normalize_identifiers {
        let key = if quoted {
            format!("\"{value}\"")
        } else {
            upper
        };
        return placeholder(names, key);
    }
    if quoted {
        format!("\"{value}\"")
    } else {
        value.to_ascii_lowercase()
    }
}

fn placeholder(names: &mut Names, key: String) -> String {
    if let Some(existing) = names.assigned.get(&key) {
        return existing.clone();
    }
    names.next += 1;
    let assigned = format!("ID{}", names.next);
    names.assigned.insert(key, assigned.clone());
    assigned
}
