use super::SqlCall;
use crate::codebase::dependencies::extract::ValueExpression;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Provenance {
    ProcessEnv(String),
    Named(String),
}

pub(crate) fn interpolations(call: &SqlCall<'_>) -> BTreeSet<Provenance> {
    let mut out = BTreeSet::new();
    let Some(source) = call.source else {
        return out;
    };
    let start = call.span.0 as usize;
    let end = (call.span.1 as usize).min(source.len());
    if start < end {
        scan_source(&source[start..end], &mut out);
    }
    out
}

pub(crate) fn is_process_env(expr: &ValueExpression) -> bool {
    process_env_key(expr).is_some()
}

fn process_env_key(expr: &ValueExpression) -> Option<&str> {
    let ValueExpression::Member(inner, key) = expr else {
        return None;
    };
    let ValueExpression::Member(root, env) = inner.as_ref() else {
        return None;
    };
    if env != "env" {
        return None;
    }
    match root.as_ref() {
        ValueExpression::Named { name, .. } if name == "process" => Some(key.as_str()),
        _ => None,
    }
}

fn scan_source(source: &str, out: &mut BTreeSet<Provenance>) {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' {
            let start = i + 2;
            if let Some(end) = matching_brace(source, start) {
                classify_interpolation(source[start..end].trim(), out);
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
}

fn matching_brace(source: &str, start: usize) -> Option<usize> {
    let mut depth = 1;
    for (offset, character) in source[start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn classify_interpolation(body: &str, out: &mut BTreeSet<Provenance>) {
    if body.contains('(') {
        return;
    }
    let parts: Vec<&str> = body.split('.').map(str::trim).collect();
    if parts.len() >= 3 && parts[0] == "process" && parts[1] == "env" {
        out.insert(Provenance::ProcessEnv(parts[2].to_string()));
        return;
    }
    if let Some(name) = parts.first() {
        if !name.is_empty() && is_ident(name) && *name != "process" {
            out.insert(Provenance::Named((*name).to_string()));
        }
    }
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|next| next == '_' || next.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codebase::postgres::EmbeddedSqlCall;
    use std::path::PathBuf;

    #[test]
    fn source_scan_classifies_env_named_and_skips_validators() {
        let mut out = BTreeSet::new();
        scan_source(
            "query(`SELECT id FROM ${process.env.SCHEMA}.users WHERE id = ${id}`)",
            &mut out,
        );
        assert!(out.contains(&Provenance::ProcessEnv("SCHEMA".into())));
        assert!(out.contains(&Provenance::Named("id".into())));
        let mut validated = BTreeSet::new();
        scan_source(
            "query(`SELECT id FROM ${schemaOf(process.env.SCHEMA ?? \"\")}.users ${unclosed`)",
            &mut validated,
        );
        assert!(validated.is_empty());
        let source = "query(`SELECT id FROM ${tables.users}`)";
        let owned = EmbeddedSqlCall::default();
        let found = interpolations(&SqlCall {
            file: PathBuf::from("query.ts"),
            span: (0, source.len() as u32),
            call: &owned,
            source: Some(source),
        });
        assert!(found.contains(&Provenance::Named("tables".into())));
        assert!(interpolations(&SqlCall {
            file: PathBuf::from("query.ts"),
            span: (0, 4),
            call: &owned,
            source: None,
        })
        .is_empty());
    }
}
