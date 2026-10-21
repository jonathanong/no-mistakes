use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::TsFactLookup;
use std::path::Path;

pub(super) fn process_env_key(expr: &ValueExpression) -> Option<&str> {
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

pub(super) fn env_key<'a>(
    facts: &CheckFactMap,
    file: &Path,
    expr: &'a ValueExpression,
) -> Option<&'a str> {
    process_env_key(expr).or_else(|| {
        let ValueExpression::Member(inner, key) = expr else {
            return None;
        };
        match inner.as_ref() {
            ValueExpression::Named { name, .. } if imported_env(facts, file, name) => {
                Some(key.as_str())
            }
            ValueExpression::Member(root, env) if env == "env" => match root.as_ref() {
                ValueExpression::Named { name, .. } if imported_process(facts, file, name) => {
                    Some(key.as_str())
                }
                _ => None,
            },
            _ => None,
        }
    })
}

pub(super) fn env_from_chain(
    facts: &CheckFactMap,
    file: &Path,
    root: &ValueExpression,
    members: &[String],
) -> Option<String> {
    let ValueExpression::Named { name, .. } = root else {
        return None;
    };
    if members.first().map(String::as_str) == Some("env")
        && members.len() >= 2
        && (name == "process" || imported_process(facts, file, name))
    {
        return Some(members[1].clone());
    }
    if imported_env(facts, file, name) {
        return members.first().cloned();
    }
    None
}

pub(crate) fn is_process_specifier(specifier: &str) -> bool {
    specifier == "node:process" || specifier == "process"
}

fn imported_env(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    imported_from_process(facts, file, local, &["env"])
}

fn imported_process(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    imported_from_process(facts, file, local, &["process", "default", "*"])
}

fn imported_from_process(facts: &CheckFactMap, file: &Path, local: &str, names: &[&str]) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.imported_bindings.iter().any(|binding| {
            !binding.is_type_only
                && binding.local == local
                && names.contains(&binding.imported.as_str())
                && is_process_specifier(&binding.specifier)
        })
    })
}
