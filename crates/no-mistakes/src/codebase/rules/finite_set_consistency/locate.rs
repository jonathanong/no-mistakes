//! Best-effort 1-based source line of the key or declaration a set was read
//! from, so findings point at the compared YAML/JSON key or TS declaration
//! instead of line 1. Falls back to the closest ancestor found, then 1.

use super::SetSpec;
use regex::Regex;

pub(super) fn set_line(spec: &SetSpec, source: &str) -> usize {
    match spec.kind.as_str() {
        "yaml-sequence" => {
            let path = spec.key.split('.').map(String::from).collect::<Vec<_>>();
            yaml_key_line(source, &path)
        }
        "yaml-string-selector" => {
            super::yaml::selector_key_path(&spec.key).map_or(1, |path| yaml_key_line(source, &path))
        }
        "ts-string-union"
        | "ts-const-object-keys"
        | "ts-const-object-property"
        | "ts-array-literal"
        | "ts-const-array-property"
        | "sql-enum" => declaration_line(source, &spec.target),
        _ => 1,
    }
}

fn declaration_line(source: &str, target: &str) -> usize {
    if target.is_empty() {
        return 1;
    }
    let name = regex::escape(target);
    let decl = format!(r"\b(type|const|let|var|enum|interface|function|class)\s+{name}\b");
    [decl, format!(r"\b{name}\b")]
        .iter()
        .map(|pattern| Regex::new(pattern).expect("escaped target forms a valid pattern"))
        .find_map(|re| source.lines().position(|line| re.is_match(line)))
        .map_or(1, |index| index + 1)
}

/// Lines that belong to the value opened by the previously matched segment.
#[derive(Clone, Copy)]
struct Region {
    /// First line to search (inclusive).
    start: usize,
    /// Indent of the line that opened the region.
    base: usize,
    /// A list may sit at the same indent as its parent key.
    allow_sibling_dash: bool,
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_content(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty() && !trimmed.starts_with('#')
}

fn is_dash(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed == "-" || trimmed.starts_with("- ")
}

fn inside(line: &str, region: Option<Region>) -> bool {
    region.is_none_or(|region| {
        indent(line) > region.base
            || (region.allow_sibling_dash && indent(line) == region.base && is_dash(line))
    })
}

fn key_matches(line: &str, key: &str) -> bool {
    let mut rest = line.trim_start();
    while let Some(next) = rest.strip_prefix("- ") {
        rest = next.trim_start();
    }
    [
        format!("{key}:"),
        format!("\"{key}\":"),
        format!("'{key}':"),
    ]
    .iter()
    .any(|prefix| rest.starts_with(prefix.as_str()))
}

fn yaml_key_line(source: &str, path: &[String]) -> usize {
    let lines = source.lines().collect::<Vec<_>>();
    let mut region = None;
    let mut found = 1;
    for segment in path {
        let hit = match segment.parse::<usize>() {
            Ok(nth) => nth_item(&lines, region, nth).map(|at| (at, false)),
            Err(_) => find_key(&lines, region, segment).map(|at| (at, true)),
        };
        let Some((at, is_key)) = hit else {
            break;
        };
        found = at + 1;
        region = Some(Region {
            start: if is_key { at + 1 } else { at },
            base: indent(lines[at]),
            allow_sibling_dash: is_key,
        });
    }
    found
}

fn region_lines<'a>(
    lines: &'a [&'a str],
    region: Option<Region>,
) -> impl Iterator<Item = (usize, &'a str)> + 'a {
    let start = region.map_or(0, |region| region.start);
    lines
        .iter()
        .copied()
        .enumerate()
        .skip(start)
        .filter(|(_, line)| is_content(line))
        .take_while(move |(at, line)| *at == start || inside(line, region))
}

fn find_key(lines: &[&str], region: Option<Region>, key: &str) -> Option<usize> {
    region_lines(lines, region)
        .find(|(_, line)| key_matches(line, key))
        .map(|(at, _)| at)
}

fn nth_item(lines: &[&str], region: Option<Region>, nth: usize) -> Option<usize> {
    let mut list_indent = None;
    region_lines(lines, region)
        .filter(|(_, line)| {
            is_dash(line) && *list_indent.get_or_insert(indent(line)) == indent(line)
        })
        .nth(nth)
        .map(|(at, _)| at)
}
