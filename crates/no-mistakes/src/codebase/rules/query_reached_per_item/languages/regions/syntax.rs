use super::Header;

pub(super) fn brace_header(line: &str) -> Option<Header> {
    if !line.contains('{') {
        return None;
    }
    let trimmed = line.trim();
    let word = first_word(trimmed);
    if matches!(word, "for" | "foreach" | "while") || each_line(trimmed) {
        return Some(Header::Loop(construct_label(trimmed)));
    }
    let paren = header_paren(line)?;
    let name = last_ident(&line[..paren]);
    if name.is_empty()
        || matches!(
            name.as_str(),
            "if" | "for" | "while" | "switch" | "catch" | "foreach"
        )
    {
        return None;
    }
    Some(Header::Function(name))
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
    let word = first_word(trimmed);
    matches!(word, "for" | "while").then(|| Header::Loop(construct_label(trimmed)))
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
    if primary > 0 { primary } else { dos }
}

fn conditional_openers(line: &str) -> usize {
    let head = |text: &str| {
        ["if", "unless"].iter().any(|word| {
            text.trim_start().strip_prefix(*word).is_some_and(|rest| {
                rest.chars().next().is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
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

pub(super) fn calls(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'(' {
            continue;
        }
        let callee = callee_before(line, index);
        if callee.is_empty() || is_keyword(&callee) {
            continue;
        }
        found.push(callee);
    }
    found
}

pub(crate) fn split_separator(callee: &str) -> Option<(&str, &str)> {
    let mut best = None;
    for (index, _) in callee.char_indices() {
        if callee[index..].starts_with('.') {
            best = Some((index, 1));
        } else if callee[index..].starts_with("::") || callee[index..].starts_with("->") {
            best = Some((index, 2));
        }
    }
    let (at, len) = best?;
    Some((&callee[..at], &callee[at + len..]))
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
    words(trimmed).iter().any(|word| *word == "each" || word.eq_ignore_ascii_case("foreach"))
        && (trimmed.contains(" do") || trimmed.contains('{'))
}

fn header_paren(line: &str) -> Option<usize> {
    let paren = line.find('(')?;
    if last_ident(&line[..paren]) != "func" {
        return Some(paren);
    }
    let close = paren + line[paren..].find(')')?;
    Some(close + 1 + line[close + 1..].find('(')?)
}

fn callee_before(line: &str, paren: usize) -> String {
    let bytes = line.as_bytes();
    let mut start = paren;
    while start > 0 {
        let ch = bytes[start - 1];
        if ch.is_ascii_alphanumeric()
            || matches!(ch, b'_' | b'.' | b':' | b'\\' | b'$' | b'>' | b'-')
        {
            start -= 1;
        } else {
            break;
        }
    }
    line[start..paren]
        .trim_start_matches(['.', ':', '\\', '>', '-'])
        .to_string()
}

fn is_keyword(callee: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "for", "foreach", "while", "if", "switch", "catch", "func", "fn", "function", "def",
        "fun", "return", "elif", "else", "do", "end", "class", "new",
    ];
    let tail = split_separator(callee).map(|(_, tail)| tail).unwrap_or(callee);
    KEYWORDS.contains(&callee) || KEYWORDS.contains(&tail)
}

fn words(line: &str) -> Vec<&str> {
    line.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|word| !word.is_empty())
        .collect()
}

fn first_word(trimmed: &str) -> &str {
    trimmed
        .split(|ch: char| ch.is_whitespace() || ch == '(' || ch == '{')
        .next()
        .unwrap_or("")
}

fn leading_ident(rest: &str) -> &str {
    let end = rest
        .char_indices()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_')
        .map(|(index, _)| index)
        .unwrap_or(rest.len());
    &rest[..end]
}

fn last_ident(before: &str) -> String {
    let mut last = String::new();
    let mut current = String::new();
    for ch in before.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            last = std::mem::take(&mut current);
        }
    }
    if current.is_empty() { last } else { current }
}
