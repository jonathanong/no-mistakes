use super::compile::{Compiled, TargetMode};
use super::singular::{join_or, last_token, singular_name};
use crate::codebase::postgres::{CatalogColumn, CatalogForeignKey, CatalogTable};
use regex::Captures;

pub(super) fn foreign_key_texts(
    column: &CatalogColumn,
    table: &CatalogTable,
    compiled: &Compiled,
) -> Vec<String> {
    let mut texts = Vec::new();
    for foreign_key in &table.foreign_keys {
        if foreign_key.columns.len() != 1 || foreign_key.columns[0] != column.name {
            continue;
        }
        if column.name == "id" {
            continue;
        }
        if foreign_key.referenced_table == table.name && !compiled.check_self {
            continue;
        }
        if let Some(text) = one_foreign_key(column, foreign_key, compiled) {
            texts.push(text);
        }
    }
    texts
}

fn one_foreign_key(
    column: &CatalogColumn,
    foreign_key: &CatalogForeignKey,
    compiled: &Compiled,
) -> Option<String> {
    let referenced = foreign_key.referenced_columns.first()?.as_str();
    if let Some(suffixes) = compiled.suffixes.get(&foreign_key.referenced_table) {
        if !suffixes.iter().any(|suffix| column.name.ends_with(suffix)) {
            let example = suffix_example(&column.name, &suffixes[0]);
            return Some(format!(
                "foreign key to {} must end in {} (for example {example})",
                foreign_key.referenced_table,
                join_or(suffixes)
            ));
        }
        return None;
    }
    let target = target_name(&foreign_key.referenced_table, compiled);
    let key = format!("_{referenced}");
    if descriptive(&column.name, referenced, &target, &compiled.target_mode) {
        return None;
    }
    let base = column.name.strip_suffix(&key).unwrap_or(&column.name);
    let (must, example) = suggestion(base, &target, &key, &compiled.target_mode);
    let passes = match compiled.target_mode {
        TargetMode::Off => return None,
        TargetMode::LastWord => last_token(base) == last_token(&target),
        TargetMode::FullName => base == target || base.ends_with(&format!("_{target}")),
    };
    if passes && column.name.ends_with(&key) {
        return None;
    }
    Some(format!(
        "foreign key to {} must end in {must} (for example {example})",
        foreign_key.referenced_table
    ))
}

fn descriptive(column: &str, referenced: &str, target: &str, mode: &TargetMode) -> bool {
    if referenced == "id" {
        return false;
    }
    let tokens = referenced
        .split('_')
        .filter(|part| !part.is_empty())
        .count();
    let repeats = column == referenced || column.ends_with(&format!("_{referenced}"));
    tokens >= 2 && repeats && names_target(referenced, target, mode)
}

fn names_target(referenced: &str, target: &str, mode: &TargetMode) -> bool {
    let stem = referenced.strip_suffix("_id").unwrap_or(referenced);
    match mode {
        TargetMode::Off => false,
        TargetMode::LastWord => {
            let target_word = last_token(target);
            last_token(stem) == target_word || target_prefix(stem, target_word)
        }
        TargetMode::FullName => {
            stem == target || stem.ends_with(&format!("_{target}")) || target_prefix(stem, target)
        }
    }
}

fn target_prefix(stem: &str, target: &str) -> bool {
    let mut stem_words = stem.split('_');
    target
        .split('_')
        .all(|target_word| stem_words.next() == Some(target_word))
}

pub(super) fn suggestion(
    base: &str,
    target: &str,
    key: &str,
    mode: &TargetMode,
) -> (String, String) {
    let want = match mode {
        TargetMode::FullName => target.to_string(),
        _ => last_token(target).to_string(),
    };
    let must = format!("{want}{key}");
    let example = if last_token(base) == last_token(target) {
        let prefix = base
            .rsplit_once('_')
            .map(|(prefix, _)| prefix)
            .unwrap_or("");
        if prefix.is_empty() {
            must.clone()
        } else {
            format!("{prefix}_{want}{key}")
        }
    } else if base.is_empty() {
        must.clone()
    } else {
        format!("{base}_{want}{key}")
    };
    (must, example)
}

fn suffix_example(column: &str, suffix: &str) -> String {
    let stem = column.strip_suffix("_id").unwrap_or(column);
    format!("{stem}{suffix}")
}

fn target_name(table: &str, compiled: &Compiled) -> String {
    for entry in &compiled.target_names {
        if let Some(captures) = entry.pattern.captures(table) {
            return substitute(&entry.name, &captures);
        }
    }
    singular_name(
        table.rsplit('.').next().unwrap_or(table),
        &compiled.singular,
    )
}

fn substitute(template: &str, captures: &Captures<'_>) -> String {
    let mut out = String::new();
    let chars: Vec<char> = template.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '$'
            && chars
                .get(index + 1)
                .is_some_and(|digit| digit.is_ascii_digit())
        {
            let group = chars[index + 1].to_digit(10).expect("ascii digit") as usize;
            if (1..=9).contains(&group) {
                if let Some(value) = captures.get(group) {
                    out.push_str(value.as_str());
                }
                index += 2;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}
