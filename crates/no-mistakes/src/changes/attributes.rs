use jsonc_parser::ast::{Object, Value};
use jsonc_parser::common::Ranged;
use serde_json::value::RawValue;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn diff(old: &[u8], new: &[u8]) -> Option<Vec<String>> {
    let old_text = std::str::from_utf8(old).ok()?;
    let new_text = std::str::from_utf8(new).ok()?;
    let old = parse_object(old_text)?;
    let new = parse_object(new_text)?;
    let mut diff = AttributeDiff {
        old_text,
        new_text,
        paths: Vec::new(),
    };
    diff.compare_objects(&old, &new, &mut String::new())?;
    diff.paths.sort_unstable();
    (!diff.paths.is_empty()).then_some(diff.paths)
}

pub(super) fn valid_pointer(path: &str) -> bool {
    path.starts_with('/')
        && path
            .split('~')
            .skip(1)
            .all(|suffix| suffix.starts_with(['0', '1']))
}

fn parse_object(source: &str) -> Option<Object<'_>> {
    // Validate strict JSON before using the AST to retain exact number spellings and duplicates.
    serde_json::from_str::<&RawValue>(source).ok()?;
    let options = jsonc_parser::ParseOptions {
        allow_comments: false,
        allow_trailing_commas: false,
        ..crate::config::jsonc_parse_options()
    };
    let value = jsonc_parser::parse_to_ast(source, &Default::default(), &options)
        .ok()?
        .value?;
    match value {
        Value::Object(object) if !has_duplicate_keys_in_object(&object) => Some(object),
        _ => None,
    }
}

fn has_duplicate_keys_in_object(object: &Object<'_>) -> bool {
    let mut seen = BTreeSet::new();
    object
        .properties
        .iter()
        .any(|prop| !seen.insert(prop.name.as_str()) || has_duplicate_keys(&prop.value))
}

fn has_duplicate_keys(value: &Value<'_>) -> bool {
    match value {
        Value::Object(object) => has_duplicate_keys_in_object(object),
        Value::Array(array) => array.elements.iter().any(has_duplicate_keys),
        _ => false,
    }
}

fn index<'a, 'source>(object: &'a Object<'source>) -> BTreeMap<&'a str, &'a Value<'source>> {
    object
        .properties
        .iter()
        .map(|prop| (prop.name.as_str(), &prop.value))
        .collect()
}

struct AttributeDiff<'a> {
    old_text: &'a str,
    new_text: &'a str,
    paths: Vec<String>,
}

impl AttributeDiff<'_> {
    fn compare_objects(
        &mut self,
        old: &Object<'_>,
        new: &Object<'_>,
        path: &mut String,
    ) -> Option<()> {
        let old_entries = index(old);
        let new_entries = index(new);
        if old_entries.is_empty() && new_entries.is_empty() {
            return Some(());
        }
        if !old_entries.keys().any(|key| new_entries.contains_key(key)) {
            if path.is_empty() {
                return None;
            }
            self.paths.push(path.clone());
            return Some(());
        }
        let old_order = old
            .properties
            .iter()
            .map(|prop| prop.name.as_str())
            .filter(|key| new_entries.contains_key(key));
        let new_order = new
            .properties
            .iter()
            .map(|prop| prop.name.as_str())
            .filter(|key| old_entries.contains_key(key));
        if old_order.ne(new_order) {
            return None;
        }
        for (key, value) in &old_entries {
            let parent = path.len();
            push_key(path, key);
            match new_entries.get(key) {
                Some(new_value) => self.compare_values(value, new_value, path)?,
                None => self.paths.push(path.clone()),
            }
            path.truncate(parent);
        }
        for key in new_entries
            .keys()
            .filter(|key| !old_entries.contains_key(*key))
        {
            let parent = path.len();
            push_key(path, key);
            self.paths.push(path.clone());
            path.truncate(parent);
        }
        Some(())
    }

    fn compare_values(
        &mut self,
        old: &Value<'_>,
        new: &Value<'_>,
        path: &mut String,
    ) -> Option<()> {
        if let (Value::Object(old), Value::Object(new)) = (old, new) {
            return self.compare_objects(old, new, path);
        }
        if std::mem::discriminant(old) != std::mem::discriminant(new) {
            return None;
        }
        if old.text(self.old_text) != new.text(self.new_text) {
            self.paths.push(path.clone());
        }
        Some(())
    }
}

fn push_key(path: &mut String, key: &str) {
    path.push('/');
    for character in key.chars() {
        match character {
            '~' => path.push_str("~0"),
            '/' => path.push_str("~1"),
            character => path.push(character),
        }
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
