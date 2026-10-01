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
    let post = Regex::new(&suffix_regex(&flags, post))
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
    let bounds = super::pattern_bounds::bounds(raw);
    if !bounds.anchors {
        bail!("postgres-object-naming option patterns.{kind}: a pattern with {{table}} must start with ^ and end with $");
    }
    if !placeholder_is_plain(raw) {
        bail!("postgres-object-naming option patterns.{kind}: {{table}} must not be inside a group, a character class or an alternation");
    }
    if bounds.assertion {
        bail!("postgres-object-naming option patterns.{kind}: {{table}} must not sit next to a zero-width assertion");
    }
    Ok(())
}

fn suffix_regex(flags: &str, post: &str) -> String {
    let tail = if verbose_flags(&active_flags(&format!("{flags}{post}"))) {
        "\n"
    } else {
        ""
    };
    format!("{flags}^(?:{post}{tail})")
}

fn verbose_flags(flags: &str) -> bool {
    let mut verbose = false;
    for group in flags.split_inclusive(')') {
        let body = group.trim_matches(|character| "()?".contains(character));
        let (on, off) = body.split_once('-').unwrap_or((body, ""));
        verbose = (verbose || on.contains('x')) && !off.contains('x');
    }
    verbose
}

fn invalid(kind: &str, error: &str) -> anyhow::Error {
    anyhow::anyhow!("postgres-object-naming option patterns.{kind}: invalid regex: {error}")
}
