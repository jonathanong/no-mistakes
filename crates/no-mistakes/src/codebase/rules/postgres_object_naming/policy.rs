use regex::Regex;
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct PluralPolicy {
    pub(super) objects: Vec<String>,
    irregular: BTreeMap<String, String>,
    irregular_values: Vec<String>,
    uncountable: Vec<String>,
    non_plural: Vec<String>,
    pub(super) ignore: Vec<Regex>,
}

impl PluralPolicy {
    pub(super) fn new(
        objects: Vec<String>,
        irregular: BTreeMap<String, String>,
        uncountable: Vec<String>,
        non_plural: Vec<String>,
        ignore: Vec<Regex>,
    ) -> Self {
        let irregular_values = irregular
            .values()
            .map(|value| value.to_ascii_lowercase())
            .collect();
        let irregular = irregular
            .into_iter()
            .map(|(key, value)| (key.to_ascii_lowercase(), value))
            .collect();
        Self {
            objects,
            irregular,
            irregular_values,
            uncountable: lower_all(uncountable),
            non_plural: lower_all(non_plural),
            ignore,
        }
    }

    pub(super) fn covers(&self, kind: &str) -> bool {
        self.objects.iter().any(|object| object == kind)
    }

    pub(super) fn ignored(&self, name: &str) -> bool {
        self.ignore.iter().any(|pattern| pattern.is_match(name))
    }

    pub(super) fn findings(&self, kind_word: &str, name: &str) -> Vec<String> {
        let tokens = tokens(name);
        let Some(last) = tokens.last() else {
            return Vec::new();
        };
        let mut findings = Vec::new();
        if let Some(plural) = self.irregular.get(&last.text.to_ascii_lowercase()) {
            findings.push(format!(
                "{kind_word} name must end in a plural word; \"{}\" is singular (plural: \"{plural}\")",
                last.text
            ));
        } else if !self.is_plural(&last.text) {
            findings.push(format!(
                "{kind_word} name must end in a plural word; \"{}\" is singular",
                last.text
            ));
        }
        for token in tokens.iter().rev().skip(1).rev() {
            if self.is_plural(&token.text)
                && !self
                    .uncountable
                    .iter()
                    .any(|word| word == &token.text.to_ascii_lowercase())
            {
                findings.push(format!(
                    "only the last word of a {kind_word} name is plural; \"{}\" is plural",
                    token.text
                ));
            }
        }
        findings
    }

    fn is_plural(&self, token: &str) -> bool {
        let lower = token.to_ascii_lowercase();
        if self.uncountable.iter().any(|word| word == &lower)
            || self.irregular_values.iter().any(|word| word == &lower)
        {
            return true;
        }
        lower.ends_with('s')
            && !lower.ends_with("ss")
            && !lower.ends_with("us")
            && !lower.ends_with("is")
            && !self.non_plural.iter().any(|word| word == &lower)
    }
}

pub(super) struct Token {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) text: String,
}

pub(super) fn tokens(name: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut start = 0;
    for (index, character) in name.char_indices() {
        if character == '_' {
            if index > start {
                tokens.push(token(name, start, index));
            }
            start = index + character.len_utf8();
        }
    }
    if start < name.len() {
        tokens.push(token(name, start, name.len()));
    }
    tokens
}

fn token(name: &str, start: usize, end: usize) -> Token {
    Token {
        start,
        end,
        text: name[start..end].to_string(),
    }
}

pub(super) fn token_inside(token: &Token, middle: Option<(usize, usize)>) -> bool {
    middle.is_some_and(|(start, end)| token.start >= start && token.end <= end)
}

pub(super) fn denied_text(token: &str, replacement: &str) -> String {
    format!("name uses denied token \"{token}\"; use \"{replacement}\"")
}

pub(super) fn spelling_text(token: &str, replacement: &str) -> String {
    format!("name spells \"{token}\"; use \"{replacement}\"")
}

pub(super) fn min_words_text(count: usize, minimum: usize, name: &str) -> String {
    let word = if count == 1 { "word" } else { "words" };
    let mut example = String::new();
    for index in 0..minimum.saturating_sub(count) {
        if index > 0 {
            example.push('_');
        }
        example.push_str(if index == 0 { "owner" } else { "part" });
    }
    if !example.is_empty() && !name.is_empty() {
        example.push('_');
    }
    example.push_str(name);
    format!(
        "table name has {count} {word}; use at least {minimum} that name the owner and the thing (for example {example})"
    )
}

pub(super) use super::name_flags::NameFlags;

pub(super) fn underscore_text(pattern: Option<&str>) -> String {
    match pattern {
        Some(pattern) => format!("\"__\" is reserved for names matching {pattern}"),
        None => "\"__\" is reserved".to_string(),
    }
}

fn lower_all(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.to_ascii_lowercase())
        .collect()
}
