use super::pattern::{compile_pattern, CompiledPattern};
use super::policy::PluralPolicy;
use super::Options;
use crate::codebase::postgres::{require_catalog_path, AllowList};
use anyhow::{bail, Result};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Compiled {
    pub(super) patterns: BTreeMap<String, CompiledPattern>,
    pub(super) check_constraint_backed_indexes: bool,
    pub(super) table_min_words: Option<usize>,
    pub(super) abbreviations: bool,
    pub(super) min_letters: usize,
    pub(super) plural: Option<PluralPolicy>,
    pub(super) denied: Vec<(String, String)>,
    pub(super) spelling: Vec<(String, String, String)>,
    pub(super) double_underscore: Option<UnderscoreRule>,
    pub(super) allow: AllowList,
    pub(super) message: Option<String>,
}

pub(super) struct UnderscoreRule {
    pub(super) raw: Option<String>,
    pub(super) allow: Option<Regex>,
}

pub(super) fn compile(options: &Options, message: Option<String>) -> Result<Compiled> {
    require_catalog_path(super::RULE_ID, options.schema_catalog_path.trim())?;
    let mut patterns = BTreeMap::new();
    for (kind, raw) in &options.patterns {
        patterns.insert(kind.clone(), compile_pattern(kind, raw)?);
    }
    let table_min_words = match options.table_min_words {
        Some(value) if value < 1 => {
            bail!("postgres-object-naming option tableMinWords: must be at least 1")
        }
        Some(value) if value > 32 => {
            bail!("postgres-object-naming option tableMinWords: must be at most 32")
        }
        Some(value) => Some(usize::try_from(value).unwrap_or(1)),
        None => None,
    };
    if options.abbreviations.min_letters < 1 {
        bail!("postgres-object-naming option abbreviations.minLetters: must be at least 1");
    }
    let denied = compile_denied(&options.denied_tokens)?;
    let spelling = super::compile_spelling::compile(&options.spelling)?;
    if let Some((token, _)) = denied.iter().find(|(token, _)| {
        spelling
            .iter()
            .any(|(key, _, _)| key.eq_ignore_ascii_case(token))
    }) {
        bail!(
            "postgres-object-naming option deniedTokens: token \"{token}\" is also a spelling key"
        );
    }
    Ok(Compiled {
        patterns,
        check_constraint_backed_indexes: options.check_constraint_backed_indexes,
        table_min_words,
        abbreviations: options.abbreviations.enabled,
        min_letters: usize::try_from(options.abbreviations.min_letters).unwrap_or(1),
        plural: compile_plural(options)?,
        denied,
        spelling,
        double_underscore: compile_underscore(&options.double_underscore)?,
        allow: AllowList::compile(super::RULE_ID, options.allow.clone())?,
        message,
    })
}

fn compile_plural(options: &Options) -> Result<Option<PluralPolicy>> {
    if options.plural.objects.is_empty() {
        bail!("postgres-object-naming option plural.objects: must not be empty");
    }
    for object in &options.plural.objects {
        if object != "table" && object != "enum" {
            bail!("postgres-object-naming option plural.objects: unknown value {object}");
        }
    }
    let mut irregular_keys = BTreeSet::new();
    for (key, value) in &options.plural.irregular_plurals {
        if key.trim().is_empty() {
            bail!("postgres-object-naming option plural.irregularPlurals: empty key");
        }
        if value.trim().is_empty() {
            bail!("postgres-object-naming option plural.irregularPlurals: empty value");
        }
        if key.eq_ignore_ascii_case(value) {
            bail!(
                "postgres-object-naming option plural.irregularPlurals: key \"{key}\" equals its value; use uncountable"
            );
        }
        if !super::policy::is_single_word(key) {
            bail!("postgres-object-naming option plural.irregularPlurals: key \"{key}\" must be a single word");
        }
        if !super::policy::is_single_word(value) {
            bail!("postgres-object-naming option plural.irregularPlurals: value \"{value}\" must be a single word");
        }
        if !irregular_keys.insert(key.to_ascii_lowercase()) {
            bail!("postgres-object-naming option plural.irregularPlurals: duplicate key {key}");
        }
        if options
            .plural
            .uncountable
            .iter()
            .any(|word| word.eq_ignore_ascii_case(key))
        {
            bail!("postgres-object-naming option plural.irregularPlurals: key \"{key}\" is uncountable");
        }
    }
    for value in options.plural.irregular_plurals.values() {
        if irregular_keys.contains(&value.to_ascii_lowercase()) {
            bail!("postgres-object-naming option plural.irregularPlurals: value \"{value}\" is also a key");
        }
    }
    require_single_words("plural.uncountable", &options.plural.uncountable)?;
    require_single_words("plural.nonPluralTokens", &options.plural.non_plural_tokens)?;
    let mut ignore = Vec::new();
    for pattern in &options.plural.ignore_patterns {
        ignore.push(Regex::new(pattern).map_err(|error| {
            anyhow::anyhow!(
                "postgres-object-naming option plural.ignorePatterns: invalid regex: {error}"
            )
        })?);
    }
    if !options.plural.enabled {
        return Ok(None);
    }
    Ok(Some(PluralPolicy::new(
        options.plural.objects.clone(),
        options.plural.irregular_plurals.clone(),
        options.plural.uncountable.clone(),
        options.plural.non_plural_tokens.clone(),
        ignore,
    )))
}

fn require_single_words(option: &str, words: &[String]) -> Result<()> {
    for word in words {
        if !super::policy::is_single_word(word) {
            bail!("postgres-object-naming option {option}: value \"{word}\" must be a single word");
        }
    }
    Ok(())
}

fn compile_denied(tokens: &[super::DeniedToken]) -> Result<Vec<(String, String)>> {
    let mut seen = BTreeSet::new();
    let mut compiled = Vec::new();
    for token in tokens {
        if token.token.trim().is_empty() {
            bail!("postgres-object-naming option deniedTokens: empty token");
        }
        if token.replacement.trim().is_empty() {
            bail!("postgres-object-naming option deniedTokens: empty replacement");
        }
        let word = &token.token;
        if !super::policy::is_single_word(word) {
            bail!("postgres-object-naming option deniedTokens: token \"{word}\" must be a single word");
        }
        if word.eq_ignore_ascii_case(&token.replacement) {
            bail!("postgres-object-naming option deniedTokens: token \"{word}\" equals its replacement");
        }
        if !seen.insert(word.to_ascii_lowercase()) {
            bail!("postgres-object-naming option deniedTokens: duplicate token {word}");
        }
        compiled.push((token.token.clone(), token.replacement.clone()));
    }
    if let Some(replacement) = compiled.iter().find_map(|(_, replacement)| {
        seen.contains(&replacement.to_ascii_lowercase())
            .then_some(replacement.as_str())
    }) {
        bail!("postgres-object-naming option deniedTokens: replacement \"{replacement}\" is also a token");
    }
    Ok(compiled)
}

fn compile_underscore(value: &Option<super::DoubleUnderscore>) -> Result<Option<UnderscoreRule>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(raw) = value.allow_pattern.clone() else {
        return Ok(Some(UnderscoreRule {
            raw: None,
            allow: None,
        }));
    };
    let allow = Regex::new(&raw).map_err(|error| {
        anyhow::anyhow!(
            "postgres-object-naming option doubleUnderscore.allowPattern: invalid regex: {error}"
        )
    })?;
    Ok(Some(UnderscoreRule {
        raw: Some(raw),
        allow: Some(allow),
    }))
}
