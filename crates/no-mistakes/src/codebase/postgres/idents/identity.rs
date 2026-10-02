use super::ident_key;
use sqlparser::ast::{ObjectName, ObjectNamePart};

pub(crate) fn parse_relation_identity(value: &str) -> Option<String> {
    let dialect = sqlparser::dialect::PostgreSqlDialect {};
    let mut parser = sqlparser::parser::Parser::new(&dialect)
        .try_with_sql(value)
        .ok()?;
    let name = parser.parse_object_name(false).ok()?;
    matches!(parser.peek_token().token, sqlparser::tokenizer::Token::EOF)
        .then(|| object_name_identity(&name))
}

pub(crate) fn object_name_identity(name: &ObjectName) -> String {
    name.0
        .iter()
        .filter_map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(relation_part_key(&ident_key(ident))),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn relation_part_key(value: &str) -> String {
    if value.contains(['.', '"'])
        || value
            .chars()
            .any(|character| character.is_ascii_uppercase())
    {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(crate) fn relation_suffix_key(value: &str) -> &str {
    let mut quoted = false;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        if character == '"' {
            quoted = !quoted;
        }
        if character == '.' && !quoted {
            start = index + 1;
        }
    }
    &value[start..]
}

pub(crate) fn relation_suffix_name(value: &str) -> String {
    let suffix = relation_suffix_key(value);
    if suffix.starts_with('"') && suffix.ends_with('"') {
        suffix[1..suffix.len() - 1].replace("\"\"", "\"")
    } else {
        suffix.to_string()
    }
}

/// Resolve an already canonical key, retaining absent external relations.
pub(crate) fn resolve_relation_key<'a>(
    keys: impl Iterator<Item = &'a str>,
    key: &str,
) -> Option<String> {
    let keys: Vec<_> = keys.collect();
    if relation_suffix_key(key) == key {
        let temporary = format!("pg_temp.{key}");
        if keys.contains(&temporary.as_str()) {
            return Some(temporary);
        }
    }
    if keys.contains(&key) {
        return Some(key.to_string());
    }
    if relation_suffix_key(key) == key {
        let mut candidates = keys
            .into_iter()
            .filter(|candidate| relation_suffix_key(candidate) == key);
        if let Some(candidate) = candidates.next() {
            return candidates.next().is_none().then(|| candidate.to_string());
        }
    }
    Some(key.to_string())
}
