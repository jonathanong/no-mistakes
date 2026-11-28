use super::lex::{first_word, last_ident, leading_ident, words};
use super::Header;

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
    let name = last_ident(&line[..paren]);
    if name.is_empty() || !function_declarator(line, paren, &name) {
        None
    } else {
        Some(Header::Function(name))
    }
}

fn function_declarator(line: &str, paren: usize, name: &str) -> bool {
    let before = &line[..paren];
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

pub(super) fn block_header(line: &str) -> Option<Header> {
    let trimmed = line.trim();
    let word = first_word(trimmed);
    if matches!(word, "for" | "while") || each_line(trimmed) {
        return Some(Header::Loop(construct_label(trimmed)));
    }
    if matches!(word, "def" | "fun") {
        let name = leading_ident(trimmed.trim_start_matches(word).trim_start());
        if !name.is_empty() {
            return Some(Header::Function(name.to_string()));
        }
    }
    None
}

pub(super) fn opener_count(line: &str) -> usize {
    let mut primary = conditional_openers(line);
    let mut dos = 0usize;
    for word in words(line) {
        match word {
            "do" => dos += 1,
            "defmodule" | "class" | "module" | "case" | "cond" | "fn" | "begin" | "def" | "fun"
            | "for" | "while" | "each" => primary += 1,
            _ => {}
        }
    }
    if primary > 0 {
        primary
    } else {
        dos
    }
}

fn conditional_openers(line: &str) -> usize {
    let head = |text: &str| {
        ["if", "unless"].iter().any(|word| {
            text.trim_start().strip_prefix(*word).is_some_and(|rest| {
                rest.chars()
                    .next()
                    .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
            })
        })
    };
    if head(line) {
        1
    } else {
        line.rfind('=')
            .is_some_and(|eq| !line[..eq].ends_with(['=', '!', '<', '>']) && head(&line[eq + 1..]))
            .into()
    }
}

pub(super) fn end_count(line: &str) -> usize {
    words(line).iter().filter(|word| **word == "end").count()
}

pub(super) fn brace_block(line: &str) -> bool {
    line.contains('{') && words(line).iter().all(|word| *word != "do")
}

fn opens_control(trimmed: &str) -> bool {
    let word = first_word(trimmed.trim_start_matches(['}', '{']).trim_start());
    matches!(
        word,
        "if" | "else"
            | "elif"
            | "elsif"
            | "elseif"
            | "switch"
            | "catch"
            | "for"
            | "while"
            | "foreach"
            | "return"
            | "do"
    )
}

fn construct_label(trimmed: &str) -> &'static str {
    match first_word(trimmed) {
        "while" => "while",
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
