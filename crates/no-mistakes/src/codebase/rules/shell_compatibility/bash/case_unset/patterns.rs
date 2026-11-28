use super::super::super::lex::{identifier, Word};
use super::super::assign::{has_expansion, indexed_name, single_word};
use crate::fx::{FxHashMap, FxHashSet};

pub(in super::super) fn extract_single_var(raw: &str) -> Option<&str> {
    let inner = raw
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw);
    let rest = inner.strip_prefix('$')?;
    let name = rest
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(rest);
    identifier(name).then_some(name)
}

fn extract_case_transform(raw: &str) -> Option<&str> {
    let inner = raw
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw);
    let body = inner.strip_prefix("${")?.strip_suffix('}')?;
    let name = ["^^", ",,", "~~", "^", ",", "~"]
        .into_iter()
        .find_map(|op| body.strip_suffix(op))?;
    identifier(name).then_some(name)
}

pub(in super::super) fn extract_derivation(rhs: &str) -> Option<(String, bool)> {
    if let Some(var) = extract_single_var(rhs) {
        return Some((var.to_owned(), true));
    }
    extract_case_transform(rhs).map(|var| (var.to_owned(), false))
}

pub(in super::super) fn subject_vars_from_header(
    rest: &[&Word<'_>],
    derived_from: &FxHashMap<String, (String, bool)>,
) -> Vec<(String, bool)> {
    let Some(subject) = rest.first() else {
        return Vec::new();
    };
    let Some((mut cur, mut exact)) = extract_derivation(subject.raw) else {
        return Vec::new();
    };
    let mut out = vec![(cur.clone(), exact)];
    for _ in 0..8 {
        let Some((parent, step_exact)) = derived_from.get(&cur) else {
            break;
        };
        exact &= *step_exact;
        if out.iter().any(|(name, _)| name == parent) {
            break;
        }
        out.push((parent.clone(), exact));
        cur = parent.clone();
    }
    out
}

pub(super) fn literal_case_patterns(words: &[Word<'_>]) -> Option<FxHashSet<String>> {
    let mut tokens: Vec<&Word<'_>> = words.iter().filter(|w| !w.redirect).collect();
    if tokens.first().is_some_and(|w| w.raw == "(") {
        tokens.remove(0);
    }
    let mut names = FxHashSet::default();
    let mut current: Option<&Word<'_>> = None;
    let mut count = 0usize;
    for word in tokens {
        if word.raw == "|" {
            push_literal_pattern(current.take(), count, &mut names)?;
            count = 0;
        } else if matches!(word.raw, "(" | ")") {
            return None;
        } else {
            current = Some(word);
            count += 1;
        }
    }
    push_literal_pattern(current, count, &mut names)?;
    Some(names)
}

fn push_literal_pattern(
    word: Option<&Word<'_>>,
    count: usize,
    names: &mut FxHashSet<String>,
) -> Option<()> {
    let word = word.filter(|_| count == 1)?;
    if word.dynamic || has_expansion(word.raw) {
        return None;
    }
    let unquoted = single_word(word.raw)?;
    if let Some(base) = indexed_name(&unquoted) {
        names.insert(base.to_owned());
    } else if unquoted.contains('[') {
        return None;
    } else if identifier(&unquoted) {
        names.insert(unquoted);
    }
    Some(())
}

pub(in super::super) fn has_catchall(words: &[Word<'_>]) -> bool {
    let mut words: Vec<&str> = words
        .iter()
        .filter(|word| !word.redirect)
        .map(|word| word.raw)
        .collect();
    if words.first() == Some(&"(") {
        words.remove(0);
    }
    let mut depth = 0usize;
    let mut alt_len = 0usize;
    let mut alt_star = false;
    for raw in words {
        match raw {
            "(" => {
                depth += 1;
                alt_len += 1;
                alt_star = false;
            }
            ")" => {
                depth = depth.saturating_sub(1);
                alt_len += 1;
                alt_star = false;
            }
            "|" if depth == 0 => {
                if alt_len == 1 && alt_star {
                    return true;
                }
                alt_len = 0;
                alt_star = false;
            }
            _ => {
                alt_len += 1;
                alt_star = alt_len == 1 && !raw.is_empty() && raw.bytes().all(|byte| byte == b'*');
            }
        }
    }
    alt_len == 1 && alt_star
}
