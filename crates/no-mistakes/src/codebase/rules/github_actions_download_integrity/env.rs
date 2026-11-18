use super::words;
use crate::fx::{fx_set, FxHashSet};
use serde_yaml::Mapping;

pub(super) struct UrlBindings<'a> {
    values: &'a Mapping,
    shadowed: FxHashSet<String>,
}

impl<'a> UrlBindings<'a> {
    pub(super) fn new(values: &'a Mapping, script: &str, parsed: &words::Parse) -> Option<Self> {
        if script.contains("${{") {
            return None;
        }
        let mut shadowed = fx_set();
        for command in &parsed.commands {
            if words::command_at(&command.words)
                .is_some_and(|(name, _)| matches!(name.as_str(), "." | "source" | "eval"))
            {
                return None;
            }
            for word in &command.words {
                let key = word
                    .text
                    .split_once('=')
                    .map_or(word.text.as_str(), |(key, _)| key);
                let key = key
                    .strip_prefix("${")
                    .unwrap_or(key)
                    .trim_end_matches([':', '+']);
                let key = key.split_once('[').map_or(key, |(name, _)| name);
                if values.contains_key(key) {
                    shadowed.insert(key.to_owned());
                }
            }
        }
        Some(Self { values, shadowed })
    }

    pub(super) fn resolve(&self, word: &words::Word, operand: &str) -> Option<&'a str> {
        if word.quoted_expansion != Some((word.text.len() - operand.len(), operand.len())) {
            return None;
        }
        let reference = operand.strip_prefix('$')?;
        let name = reference
            .strip_prefix('{')
            .and_then(|braced| braced.strip_suffix('}'))
            .unwrap_or(reference);
        if !valid_name(name) || self.shadowed.contains(name) {
            return None;
        }
        let value = self.values.get(name)?.as_str()?;
        if value.contains(['$', '`', '{', '}']) || value.contains(char::is_whitespace) {
            return None;
        }
        Some(value)
    }
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
