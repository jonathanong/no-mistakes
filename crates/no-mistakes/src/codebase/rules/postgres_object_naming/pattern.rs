use super::expand::{middle_matches, suggestion};
use super::pattern_walk::{active_flags, placeholder_is_plain, walk_placeholders};
use anyhow::{bail, Result};
use regex::Regex;

const KINDS: &[&str] = &[
    "index",
    "uniqueIndex",
    "trigger",
    "function",
    "triggerFunction",
    "view",
    "materializedView",
    "table",
    "column",
    "enum",
];

pub(super) struct CompiledPattern {
    pub(super) raw: String,
    kind: PatternBody,
}

enum PatternBody {
    Plain(Regex),
    Table { pre: Regex, post: Regex },
}

pub(super) struct NameMatch {
    pub(super) middle: Option<(usize, usize)>,
}

pub(super) fn compile_pattern(kind: &str, raw: &str) -> Result<CompiledPattern> {
    if !KINDS.contains(&kind) {
        bail!("postgres-object-naming option patterns.{kind}: unknown pattern kind");
    }
    let found = walk_placeholders(raw);
    if found.offsets.is_empty() {
        let regex = Regex::new(raw).map_err(|error| invalid(kind, &error.to_string()))?;
        return Ok(CompiledPattern {
            raw: raw.to_string(),
            kind: PatternBody::Plain(regex),
        });
    }
    validate_placeholder(kind, raw, found.offsets.len())?;
    let at = found.offsets[0];
    let pre = &raw[..at];
    let post = &raw[at + "{table}".len()..];
    let flags = active_flags(pre);
    let pre = Regex::new(&format!("{pre}$")).map_err(|error| invalid(kind, &error.to_string()))?;
    let post = Regex::new(&format!("{flags}^(?:{post})"))
        .map_err(|error| invalid(kind, &error.to_string()))?;
    Ok(CompiledPattern {
        raw: raw.to_string(),
        kind: PatternBody::Table { pre, post },
    })
}

pub(super) fn match_name(
    pattern: &CompiledPattern,
    name: &str,
    table: Option<&str>,
    abbreviations: bool,
    min_letters: usize,
) -> Option<NameMatch> {
    match &pattern.kind {
        PatternBody::Plain(regex) => regex.is_match(name).then_some(NameMatch { middle: None }),
        PatternBody::Table { pre, post } => {
            let table = table?;
            let bounds = bounds(name);
            let mut best = None;
            for start in bounds
                .iter()
                .copied()
                .filter(|index| covers(pre, &name[..*index]))
            {
                for end in bounds
                    .iter()
                    .copied()
                    .filter(|index| covers(post, &name[*index..]))
                {
                    if start <= end
                        && middle_matches(&name[start..end], table, abbreviations, min_letters)
                    {
                        best = Some(prefer(best, start, end));
                    }
                }
            }
            best.map(|(start, end)| NameMatch {
                middle: Some((start, end)),
            })
        }
    }
}

pub(super) fn table_note(
    pattern: &CompiledPattern,
    table: &str,
    abbreviations: bool,
    min_letters: usize,
) -> String {
    let PatternBody::Table { .. } = &pattern.kind else {
        return String::new();
    };
    if !abbreviations {
        return format!(" ({{table}} = {table})");
    }
    match suggestion(table, min_letters) {
        Some(hint) => format!(" ({{table}} = {table} or an abbreviation such as {hint})"),
        None => format!(" ({{table}} = {table})"),
    }
}

fn covers(regex: &Regex, text: &str) -> bool {
    regex
        .find(text)
        .is_some_and(|found| found.start() == 0 && found.end() == text.len())
}

fn prefer(best: Option<(usize, usize)>, start: usize, end: usize) -> (usize, usize) {
    match best {
        Some((current, current_end))
            if current < start || (current == start && current_end >= end) =>
        {
            (current, current_end)
        }
        _ => (start, end),
    }
}

fn bounds(name: &str) -> Vec<usize> {
    name.char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(name.len()))
        .collect()
}

fn validate_placeholder(kind: &str, raw: &str, count: usize) -> Result<()> {
    if !matches!(kind, "index" | "uniqueIndex" | "trigger") {
        bail!("postgres-object-naming option patterns.{kind}: {{table}} is only allowed in index, uniqueIndex and trigger");
    }
    if count > 1 {
        bail!("postgres-object-naming option patterns.{kind}: {{table}} may appear only once");
    }
    if !without_leading_flags(raw).starts_with('^') || !ends_unescaped_dollar(raw) {
        bail!("postgres-object-naming option patterns.{kind}: a pattern with {{table}} must start with ^ and end with $");
    }
    if !placeholder_is_plain(raw) {
        bail!("postgres-object-naming option patterns.{kind}: {{table}} must not be inside a group, a character class or an alternation");
    }
    if split_touches_boundary(raw) {
        bail!("postgres-object-naming option patterns.{kind}: {{table}} must not sit next to a word-boundary assertion");
    }
    Ok(())
}

fn split_touches_boundary(raw: &str) -> bool {
    let Some(at) = walk_placeholders(raw).offsets.first().copied() else {
        return false;
    };
    let pre = &raw[..at];
    let post = &raw[at + "{table}".len()..];
    pre.ends_with("\\b")
        || pre.ends_with("\\B")
        || post.starts_with("\\b")
        || post.starts_with("\\B")
}

fn ends_unescaped_dollar(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    if bytes.last() != Some(&b'$') {
        return false;
    }
    let mut slashes = 0;
    let mut index = bytes.len() - 1;
    while index > 0 && bytes[index - 1] == b'\\' {
        slashes += 1;
        index -= 1;
    }
    slashes % 2 == 0
}

fn without_leading_flags(raw: &str) -> &str {
    let mut rest = raw;
    while let Some(next) = strip_flags(rest) {
        rest = next;
    }
    rest
}

fn strip_flags(raw: &str) -> Option<&str> {
    let body = raw.strip_prefix("(?")?;
    let end = body.find(')')?;
    flag_body(&body[..end]).then_some(&body[end + 1..])
}

fn flag_body(body: &str) -> bool {
    let mut dash = false;
    let mut flag = false;
    for character in body.chars() {
        match character {
            '-' if !dash => dash = true,
            'i' | 'm' | 's' | 'u' | 'U' | 'x' | 'R' => flag = true,
            _ => return false,
        }
    }
    flag
}

fn invalid(kind: &str, error: &str) -> anyhow::Error {
    anyhow::anyhow!("postgres-object-naming option patterns.{kind}: invalid regex: {error}")
}
