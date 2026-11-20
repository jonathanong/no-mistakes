use super::words;
use crate::fx::{fx_set, FxHashSet};
use serde_yaml::Mapping;

mod writes;

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
            writes::observe(&command.words, |key| {
                if values.contains_key(key) {
                    shadowed.insert(key.to_owned());
                }
            })?;
        }
        Some(Self { values, shadowed })
    }

    pub(super) fn resolve(&self, word: &words::Word, operand: &str) -> Option<&'a str> {
        let whole = Some((word.text.len() - operand.len(), operand.len()));
        let bare = word.bare_expansion == whole;
        if word.quoted_expansion != whole && !bare {
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
        if bare && pathname_expands(value) {
            return None;
        }
        Some(value)
    }
}

fn pathname_expands(value: &str) -> bool {
    value.contains(['*', '?']) || has_closed_bracket(value)
}

fn has_closed_bracket(value: &str) -> bool {
    value
        .char_indices()
        .any(|(index, ch)| ch == '[' && value[index + 1..].contains(']'))
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
