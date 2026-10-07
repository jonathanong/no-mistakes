use super::compile::Compiled;
use super::name_flags::NameFlags;
use super::pattern::{match_name, table_note};
use super::policy::{denied_text, spelling_text, token_inside, tokens, underscore_text};
use super::RULE_ID;
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef};
use crate::codebase::rules::RuleFinding;

pub(super) struct NameCheck<'a> {
    pub(super) kind: &'a str,
    pub(super) name: &'a str,
    pub(super) object: CatalogObjectRef,
    pub(super) table: Option<&'a str>,
    pub(super) flags: NameFlags,
}

pub(super) fn consider(
    check: NameCheck<'_>,
    compiled: &Compiled,
    path: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let NameCheck {
        kind,
        name,
        object,
        table,
        flags,
    } = check;
    let pattern = compiled.patterns.get(kind);
    let matched = pattern.and_then(|pattern| {
        match_name(
            pattern,
            name,
            table,
            compiled.abbreviations,
            compiled.min_letters,
        )
    });
    if let Some(pattern) = pattern {
        if matched.is_none() {
            let note = table
                .map(|table| {
                    table_note(pattern, table, compiled.abbreviations, compiled.min_letters)
                })
                .unwrap_or_default();
            push(
                findings,
                path,
                object.clone(),
                &format!(
                    "{} name does not match pattern {}{note}",
                    kind_word(kind),
                    pattern.raw
                ),
            );
        }
    }
    if flags.tokens {
        let middle = matched.and_then(|matched| matched.middle);
        push_tokens(name, middle, table, &object, compiled, path, findings);
    }
    if let Some(plural_kind) = flags.plural {
        if let Some(plural) = &compiled.plural {
            if plural.covers(plural_kind) && !plural.ignored(name) {
                for text in plural.findings(plural_kind, name) {
                    push(findings, path, object.clone(), &text);
                }
            }
        }
    }
    if flags.underscore {
        if let Some(rule) = &compiled.double_underscore {
            let allowed = rule
                .allow
                .as_ref()
                .is_some_and(|pattern| pattern.is_match(name));
            if name.contains("__") && !allowed {
                push(
                    findings,
                    path,
                    object,
                    &underscore_text(rule.raw.as_deref()),
                );
            }
        }
    }
}

pub(super) fn unqualified(name: &str) -> String {
    let last = last_identifier(name.trim());
    let Some(quoted) = last
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return last;
    };
    quoted.replace("\"\"", "\"")
}

fn last_identifier(name: &str) -> String {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = name.chars().peekable();
    while let Some(character) = chars.next() {
        if quoted {
            current.push(character);
            if character == '"' && chars.next_if_eq(&'"').is_none() {
                quoted = false;
            } else if character == '"' {
                current.push('"');
            }
            continue;
        }
        if character == '"' {
            quoted = true;
            current.push(character);
            continue;
        }
        if character == '.' {
            parts.push(std::mem::take(&mut current));
            continue;
        }
        current.push(character);
    }
    parts.push(current);
    parts
        .into_iter()
        .rev()
        .find(|part| !part.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn push_tokens(
    name: &str,
    middle: Option<(usize, usize)>,
    table: Option<&str>,
    object: &CatalogObjectRef,
    compiled: &Compiled,
    path: &str,
    findings: &mut Vec<RuleFinding>,
) {
    for token in tokens(name) {
        if covered_by_table(&token, middle, table) {
            continue;
        }
        for (denied, replacement) in &compiled.denied {
            if token.text.eq_ignore_ascii_case(denied)
                && !super::replacement::contains(name, &token, replacement)
            {
                push(
                    findings,
                    path,
                    object.clone(),
                    &denied_text(denied, replacement),
                );
            }
        }
        for (lower, display, replacement) in &compiled.spelling {
            if token.text.eq_ignore_ascii_case(lower) {
                push(
                    findings,
                    path,
                    object.clone(),
                    &spelling_text(display, replacement),
                );
            }
        }
    }
}

pub(super) fn push(
    findings: &mut Vec<RuleFinding>,
    path: &str,
    object: CatalogObjectRef,
    text: &str,
) {
    findings.push(catalog_finding(RULE_ID, path, &object, text));
}

fn covered_by_table(
    token: &super::policy::Token,
    middle: Option<(usize, usize)>,
    table: Option<&str>,
) -> bool {
    token_inside(token, middle)
        && table.is_none_or(|table| {
            tokens(table)
                .iter()
                .any(|part| part.text.eq_ignore_ascii_case(&token.text))
        })
}

fn kind_word(kind: &str) -> &str {
    match kind {
        "materializedView" => "materialized view",
        "triggerFunction" => "trigger function",
        "uniqueIndex" => "index",
        other => other,
    }
}
