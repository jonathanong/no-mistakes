use super::{
    globs::{compile_patterns, selected_by},
    CoverageUnit, RULE_ID,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(super) struct CoveragePath {
    pub(super) rel: String,
    pub(super) synthetic: bool,
}

pub(super) fn coverage_paths(
    root: &Path,
    unit: &CoverageUnit,
    files: &[PathBuf],
) -> Result<Vec<CoveragePath>> {
    let compiled = compile_patterns(&unit.patterns)
        .with_context(|| format!("invalid glob in {RULE_ID} {}", unit.project))?;
    let mut seen = BTreeSet::new();
    let mut paths = Vec::new();

    for rel in files.iter().map(|path| relative_slash_path(root, path)) {
        if selected_by(&compiled, &rel) && seen.insert(rel.clone()) {
            paths.push(CoveragePath {
                rel,
                synthetic: false,
            });
        }
    }

    for rel in witness_paths(&unit.patterns) {
        if selected_by(&compiled, &rel) && seen.insert(rel.clone()) {
            paths.push(CoveragePath {
                rel,
                synthetic: true,
            });
        }
    }

    Ok(paths)
}

fn witness_paths(patterns: &[String]) -> Vec<String> {
    patterns
        .iter()
        .filter(|pattern| !pattern.starts_with('!'))
        .filter(|pattern| needs_recursive_witness(pattern))
        .map(|pattern| witness_path(pattern))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn needs_recursive_witness(pattern: &str) -> bool {
    pattern
        .split_once("**/")
        .is_some_and(|(_, tail)| tail.chars().any(|ch| ch != '*' && ch != '/'))
}

fn witness_path(pattern: &str) -> String {
    let mut chars = pattern.chars().peekable();
    let mut out = String::with_capacity(pattern.len() + 24);
    while let Some(ch) = chars.next() {
        match ch {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                out.push_str("__no_mistakes_witness__/nested");
            }
            '*' => out.push_str("__no_mistakes_witness__"),
            '?' => out.push('x'),
            '{' => copy_first_brace_alternative(&mut chars, &mut out),
            '[' => copy_first_class_character(&mut chars, &mut out),
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            ch => out.push(ch),
        }
    }
    out.trim_matches('/').to_string()
}

fn copy_first_brace_alternative<I>(chars: &mut std::iter::Peekable<I>, out: &mut String)
where
    I: Iterator<Item = char>,
{
    for ch in chars.by_ref() {
        match ch {
            ',' => {
                for rest in chars.by_ref() {
                    if rest == '}' {
                        break;
                    }
                }
                break;
            }
            '}' => break,
            ch => out.push(ch),
        }
    }
}

fn copy_first_class_character<I>(chars: &mut std::iter::Peekable<I>, out: &mut String)
where
    I: Iterator<Item = char>,
{
    let first = chars.next().unwrap_or('x');
    out.push(first);
    for ch in chars.by_ref() {
        if ch == ']' {
            break;
        }
    }
}
