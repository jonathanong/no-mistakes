use super::super::lex::{first_word, leading_ident, standalone_words, words};
use super::super::Header;
use super::{construct_label, each_line};

pub(in super::super) fn block_header(line: &str) -> Option<Header> {
    let trimmed = line.trim();
    let word = first_word(trimmed);
    if matches!(word, "for" | "while" | "until") || each_line(trimmed) {
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

pub(in super::super) fn opener_count(line: &str) -> usize {
    const BLOCKS: &[&str] = &[
        "defmodule",
        "class",
        "module",
        "case",
        "fn",
        "begin",
        "def",
        "fun",
        "for",
    ];
    let mut primary = conditional_openers(line) + usize::from(each_line(line.trim()));
    let mut dos = 0usize;
    for word in standalone_words(line) {
        if word == "do" {
            dos += 1;
        } else if BLOCKS.contains(&word) {
            primary += 1;
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
        ["if", "unless", "while", "until"].iter().any(|word| {
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

pub(in super::super) fn end_count(line: &str) -> usize {
    standalone_words(line)
        .iter()
        .filter(|word| **word == "end")
        .count()
}

pub(in super::super) fn brace_block_offset(line: &str) -> Option<usize> {
    if !(line.contains('{') && words(line).iter().all(|word| *word != "do")) {
        return None;
    }
    line.rfind("each")
        .and_then(|pos| line[pos..].find('{').map(|rel| pos + rel))
        .or_else(|| line.find('{'))
}
