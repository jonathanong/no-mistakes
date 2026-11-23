use super::super::url::{has_skip, skip_comment};
use super::StepRun;

pub(super) fn step_skipped(lines: &[&str], place: Option<&StepRun>, script: &str) -> bool {
    if has_skip(script) {
        return true;
    }
    let Some(place) = place else {
        return false;
    };
    // The skip comment sits above the `- ` item, which may be above `run:`.
    let start = place.item_line;
    if start >= 2 {
        let prev = lines.get(start - 2).copied().unwrap_or("");
        if prev.trim_start().starts_with('#') && has_skip(prev) {
            return true;
        }
    }
    let first_run = place.lines.first().map_or(start, |(line, _)| *line);
    lines
        .iter()
        .take(first_run)
        .skip(start.saturating_sub(1))
        .any(|line| has_skip(line))
}

pub(super) fn line_scope_attributable(
    lines: &[&str],
    place: Option<&StepRun>,
    script: &str,
) -> bool {
    let Some(place) = place else {
        return false;
    };
    let count = script.lines().count();
    let Some((first, _)) = place.lines.first() else {
        return false;
    };
    if source_line(lines, first - 1)
        .and_then(run_value)
        .is_some_and(folded_scalar)
        || count > place.lines.len()
        || place.lines[count..]
            .iter()
            .any(|(_, text)| !text.trim().is_empty())
    {
        return false;
    }
    if count == 1
        && source_line(lines, *first)
            .and_then(run_value)
            .is_some_and(|value| !value.starts_with('*'))
    {
        return true;
    }
    place.lines[..count]
        .iter()
        .zip(script.lines())
        .enumerate()
        .all(|(index, ((line, text), script_line))| {
            *line == first + index && text.trim() == script_line.trim()
        })
}

fn folded_scalar(value: &str) -> bool {
    let header = value
        .split_once('#')
        .map_or(value, |(header, _)| header)
        .trim_end();
    header.strip_prefix('>').is_some_and(|indicator| {
        indicator
            .chars()
            .all(|ch| matches!(ch, '+' | '-' | '1'..='9'))
    })
}

pub(super) fn yaml_line_skipped(lines: &[&str], place: Option<&StepRun>, line: usize) -> bool {
    let Some(place) = place else {
        return false;
    };
    if place.lines.first().is_none_or(|(first, _)| line != *first) {
        return false;
    }
    source_line(lines, line).is_some_and(|text| run_value(text).is_some_and(yaml_skip))
        || source_line(lines, line - 1).is_some_and(|text| {
            let text = text.trim_start();
            text.strip_prefix('#').is_some_and(skip_comment)
                || (place.item_line < line && run_value(text).is_some_and(yaml_skip))
        })
}

fn source_line<'a>(lines: &[&'a str], line: usize) -> Option<&'a str> {
    lines.get(line.checked_sub(1)?).copied()
}

fn run_value(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let line = line.strip_prefix("- ").map_or(line, str::trim_start);
    let (key, value) = line.split_once(':')?;
    (key.trim().trim_matches(['\'', '"']) == "run").then(|| without_anchor(value.trim_start()))
}

fn without_anchor(value: &str) -> &str {
    if value.starts_with('&') {
        value
            .split_once(char::is_whitespace)
            .map_or("", |(_, rest)| rest.trim_start())
    } else {
        value
    }
}

fn yaml_skip(value: &str) -> bool {
    let mut chars = value.char_indices().peekable();
    if let Some(quote @ ('\'' | '"')) = value.chars().next() {
        chars.next();
        while let Some((_, ch)) = chars.next() {
            if quote == '"' && ch == '\\' {
                chars.next();
            } else if ch == quote {
                if quote == '\'' && chars.next_if(|(_, next)| *next == quote).is_some() {
                    continue;
                }
                break;
            }
        }
    }
    chars
        .find(|(index, ch)| {
            *ch == '#' && (*index == 0 || value[..*index].ends_with(char::is_whitespace))
        })
        .is_some_and(|(index, _)| skip_comment(&value[index + 1..]))
}
