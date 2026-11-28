use super::lex::{first_word, last_ident, leading_ident, words};
use super::Header;

mod block;
pub(super) use block::{block_header, brace_block_offset, end_count, opener_count};

pub(super) fn brace_header(line: &str) -> Option<Header> {
    if line.contains('{') {
        bare_header(line)
    } else {
        None
    }
}

pub(super) fn bare_header(line: &str) -> Option<Header> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('{') {
        return None;
    }
    let word = first_word(trimmed);
    if matches!(word, "for" | "foreach" | "while") || each_line(trimmed) {
        return Some(Header::Loop(construct_label(trimmed)));
    }
    if opens_control(trimmed) || trimmed.ends_with(';') {
        return None;
    }
    let paren = header_paren(line)?;
    let name = last_ident(strip_generics(&line[..paren]));
    if name.is_empty() || !function_declarator(line, paren, &name) {
        None
    } else {
        Some(Header::Function(name))
    }
}

fn strip_generics(before: &str) -> &str {
    let trimmed = before.trim_end();
    let (open, close) = match trimmed.chars().next_back() {
        Some('>') => ('<', '>'),
        Some(']') => ('[', ']'),
        _ => return before,
    };
    let mut depth = 0i32;
    for (index, ch) in trimmed.char_indices().rev() {
        if ch == close {
            depth += 1;
        } else if ch == open {
            depth -= 1;
            if depth == 0 {
                return &trimmed[..index];
            }
        }
    }
    before
}

fn function_declarator(line: &str, paren: usize, name: &str) -> bool {
    let before = strip_generics(&line[..paren]);
    if before.contains('=') {
        return false;
    }
    if words(before)
        .iter()
        .any(|word| matches!(*word, "fn" | "func" | "function" | "fun" | "def"))
    {
        return true;
    }
    let Some(name_at) = before.rfind(name) else {
        return false;
    };
    name_at > 0
        && before.as_bytes()[name_at - 1].is_ascii_whitespace()
        && !words(&before[..name_at]).is_empty()
}

pub(super) fn indent_header(line: &str) -> Option<Header> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed
        .strip_prefix("async def ")
        .or_else(|| trimmed.strip_prefix("def "))
    {
        let name = leading_ident(rest);
        return (!name.is_empty()).then(|| Header::Function(name.to_string()));
    }
    let logical = async_for(trimmed);
    let word = first_word(logical);
    matches!(word, "for" | "while").then(|| Header::Loop(construct_label(logical)))
}

fn async_for(trimmed: &str) -> &str {
    if trimmed.starts_with("async for") {
        trimmed["async ".len()..].trim_start()
    } else {
        trimmed
    }
}

fn opens_control(trimmed: &str) -> bool {
    const CONTROL: &[&str] = &[
        "if", "else", "elif", "elsif", "elseif", "switch", "match", "catch", "for", "while",
        "foreach", "return", "do",
    ];
    let word = first_word(trimmed.trim_start_matches(['}', '{']).trim_start());
    CONTROL.contains(&word)
}

fn construct_label(trimmed: &str) -> &'static str {
    match first_word(trimmed) {
        "while" => "while",
        "until" => "until",
        "foreach" => "foreach",
        _ if each_line(trimmed) => "each",
        _ => "for",
    }
}

fn each_line(trimmed: &str) -> bool {
    words(trimmed)
        .iter()
        .any(|word| *word == "each" || word.eq_ignore_ascii_case("foreach"))
        && (trimmed.contains(" do") || trimmed.contains('{'))
}

pub(super) fn header_paren(line: &str) -> Option<usize> {
    let paren = line.find('(')?;
    if last_ident(&line[..paren]) != "func" {
        return Some(paren);
    }
    let close = paren + line[paren..].find(')')?;
    Some(close + 1 + line[close + 1..].find('(')?)
}
