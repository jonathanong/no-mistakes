use anyhow::Result;
use globset::{GlobBuilder, GlobMatcher};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PredicateQuantifier {
    Some,
    Every,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledGlob {
    negate: bool,
    matcher: GlobMatcher,
}

pub(super) fn compile_patterns(patterns: &[String]) -> Result<Vec<CompiledGlob>> {
    patterns
        .iter()
        .map(|pattern| {
            let (negate, glob) = split_negation(pattern);
            let matcher = GlobBuilder::new(&normalize_github_glob(glob))
                .literal_separator(true)
                .build()?
                .compile_matcher();
            Ok(CompiledGlob { negate, matcher })
        })
        .collect()
}

pub(super) fn selected_by(globs: &[CompiledGlob], path: &str) -> bool {
    let mut selected = false;
    for glob in globs {
        if glob.matcher.is_match(path) {
            selected = !glob.negate;
        }
    }
    selected
}

pub(super) fn selected_by_paths_filter(
    predicates: &[Vec<CompiledGlob>],
    quantifier: PredicateQuantifier,
    path: &str,
) -> bool {
    match quantifier {
        PredicateQuantifier::Some => predicates
            .iter()
            .any(|predicate| predicate_matches(predicate, path)),
        PredicateQuantifier::Every => {
            !predicates.is_empty()
                && predicates
                    .iter()
                    .all(|predicate| predicate_matches(predicate, path))
        }
    }
}

pub(super) fn compile_pattern_predicates(
    patterns: &[Vec<String>],
) -> Result<Vec<Vec<CompiledGlob>>> {
    patterns
        .iter()
        .map(|predicate| compile_patterns(predicate))
        .collect()
}

fn predicate_matches(predicate: &[CompiledGlob], path: &str) -> bool {
    predicate.iter().any(|glob| glob.predicate_matches(path))
}

impl CompiledGlob {
    fn predicate_matches(&self, path: &str) -> bool {
        self.matcher.is_match(path) != self.negate
    }
}

fn split_negation(pattern: &str) -> (bool, &str) {
    match pattern.strip_prefix('!') {
        Some(rest) => (true, rest),
        None => (false, pattern),
    }
}

fn normalize_github_glob(pattern: &str) -> String {
    if let Some(expanded) = expand_double_star_segment(pattern) {
        return expanded;
    }
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::with_capacity(pattern.len() + 2);
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
            out.push_str("**");
            i += 2;
            if matches!(chars.get(i), Some(c) if *c != '/') {
                out.push_str("/*");
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn expand_double_star_segment(pattern: &str) -> Option<String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i + 1 < chars.len() {
        if chars[i] == '*'
            && chars[i + 1] == '*'
            && chars.get(i + 2).is_some_and(|ch| *ch != '/')
            && (i == 0 || chars[i - 1] == '/')
        {
            let tail_start = i + 2;
            let segment_end = chars[tail_start..]
                .iter()
                .position(|ch| *ch == '/')
                .map(|offset| tail_start + offset)
                .unwrap_or(chars.len());
            let prefix: String = chars[..i].iter().collect();
            let segment_tail: String = chars[tail_start..segment_end].iter().collect();
            let suffix: String = chars[segment_end..].iter().collect();
            return Some(format!(
                "{prefix}{{*{segment_tail},**/*{segment_tail}}}{suffix}"
            ));
        }
        i += 1;
    }
    None
}
