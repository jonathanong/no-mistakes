use super::options::Options;
use super::scan::Step;
use super::{hit, RuleFinding};

pub(super) fn check(rel: &str, step: &Step, options: &Options, findings: &mut Vec<RuleFinding>) {
    for (name, field) in [("key", &step.key), ("restore-keys", &step.restore_keys)] {
        let Some(field) = field else { continue };
        // The source walker can leave YAML single quotes wrapped when the value has escapes.
        let decoded = field
            .value
            .starts_with('\'')
            .then(|| serde_yaml::from_str::<String>(&field.value).ok())
            .flatten();
        let value = decoded.as_deref().unwrap_or(&field.value);
        for pattern in literal_hash_patterns(value) {
            if is_broad(&pattern, options)
                && !options.allowed_hash_files_patterns.contains(&pattern)
            {
                findings.push(hit(
                    rel,
                    field.line,
                    format!(
                        "Cache `{name}` calls hashFiles with broad pattern `{pattern}`. Hash exact dependency paths so source changes do not invalidate the cache, or list the exact pattern in allowedHashFilesPatterns."
                    ),
                ));
            }
        }
    }
}

fn is_broad(pattern: &str, options: &Options) -> bool {
    let terminal = pattern.trim_end_matches('/').rsplit('/').next();
    terminal == Some("*") || (pattern.contains("**") && !is_recursive_lockfile(pattern, options))
}

fn is_recursive_lockfile(pattern: &str, options: &Options) -> bool {
    let Some((dirs, basename)) = pattern.rsplit_once('/') else {
        return false;
    };
    let mut parts = dirs.rsplit('/');
    parts.next() == Some("**")
        && parts.all(|part| !part.contains(['*', '?', '[', ']', '{', '}']))
        && !basename.contains(['*', '?', '[', ']', '{', '}'])
        && options.caches.iter().any(|cache| {
            super::text::norm_path(&cache.lockfile).rsplit('/').next() == Some(basename)
        })
}

fn literal_hash_patterns(mut value: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    while let Some((_, body)) = value.split_once("${{") {
        let start = patterns.len();
        let Some(rest) = scan_expression(body, &mut patterns) else {
            patterns.truncate(start);
            break;
        };
        value = rest;
    }
    patterns
}

struct Group<'a> {
    close: u8,
    hash_arg: Option<&'a str>,
}

impl<'a> Group<'a> {
    fn finish_argument(&mut self, rest: &'a str, patterns: &mut Vec<String>) {
        let Some(start) = self.hash_arg.as_mut() else {
            return;
        };
        let argument = start[..start.len() - rest.len()].trim();
        if let Some((literal, "")) = string_literal(argument) {
            patterns.push(literal.replace("''", "'"));
        }
        *start = &rest[1..];
    }
}

fn scan_expression<'a>(mut body: &'a str, patterns: &mut Vec<String>) -> Option<&'a str> {
    let mut groups: Vec<Group<'_>> = Vec::new();
    let mut hash_function = false;
    let mut member = false;
    loop {
        let token = body.trim_start();
        if let Some(rest) = token.strip_prefix("}}") {
            return groups.is_empty().then_some(rest);
        }
        let first = *token.as_bytes().first()?;
        let mut next_hash_function = false;
        body = match first {
            b'\'' => string_literal(token)?.1,
            b'"' => return None,
            b'(' | b'[' => {
                let rest = &token[1..];
                groups.push(Group {
                    close: if first == b'(' { b')' } else { b']' },
                    hash_arg: (first == b'(' && hash_function).then_some(rest),
                });
                rest
            }
            b')' | b']' => {
                let mut group = groups.pop()?;
                if group.close != first {
                    return None;
                }
                group.finish_argument(token, patterns);
                &token[1..]
            }
            b',' => {
                if let Some(group) = groups.last_mut() {
                    group.finish_argument(token, patterns);
                }
                &token[1..]
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' | b'-' => {
                let end = token
                    .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-')))
                    .unwrap_or(token.len());
                next_hash_function = !member && token[..end].eq_ignore_ascii_case("hashFiles");
                &token[end..]
            }
            _ => &token[token.chars().next()?.len_utf8()..],
        };
        hash_function = next_hash_function;
        member = first == b'.';
    }
}

fn string_literal(value: &str) -> Option<(&str, &str)> {
    let body = value.strip_prefix('\'')?;
    let mut cursor = body;
    while let Some((_, rest)) = cursor.split_once('\'') {
        if let Some(remaining) = rest.strip_prefix('\'') {
            cursor = remaining;
        } else {
            return Some((&body[..body.len() - rest.len() - 1], rest));
        }
    }
    None
}
