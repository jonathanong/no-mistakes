use super::masked_source::FileAllow;
use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

mod scan;

pub const RULE_ID: &str = "csharp-no-async-void-delegate";

pub(super) const DEFAULT_MESSAGE: &str =
    "do not pass an async lambda to a void Action API; wrap as () => _ = FooAsync()";

const DEFAULT_CONSTRUCTORS: &[&str] = &["Command"];
const DEFAULT_METHODS: &[&str] = &["BeginInvokeOnMainThread"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) constructors: Vec<String>,
    pub(crate) methods: Vec<String>,
    pub(crate) allow: Vec<String>,
    pub(crate) message: Option<String>,
}

pub(super) struct CompiledOptions {
    pub(super) constructors: Vec<regex::Regex>,
    pub(super) methods: Vec<regex::Regex>,
    pub(super) allow: FileAllow,
    pub(super) message: String,
}

super::masked_source_rule_entrypoints!(compile_options, is_csharp_file, scan::check_file);

fn compile_options(opts: Options) -> Result<CompiledOptions> {
    let constructors = if opts.constructors.is_empty() {
        constructor_regexes(DEFAULT_CONSTRUCTORS.iter().copied())
    } else {
        constructor_regexes(opts.constructors.iter().map(String::as_str))
    };
    let methods = if opts.methods.is_empty() {
        method_regexes(DEFAULT_METHODS.iter().copied())
    } else {
        method_regexes(opts.methods.iter().map(String::as_str))
    };
    Ok(CompiledOptions {
        constructors,
        methods,
        allow: FileAllow::new(&opts.allow, &format!("{RULE_ID} allow"))?,
        message: match opts.message.filter(|hint| !hint.is_empty()) {
            Some(hint) => format!("{DEFAULT_MESSAGE} {hint}"),
            None => DEFAULT_MESSAGE.to_string(),
        },
    })
}

fn constructor_regexes<'a>(names: impl Iterator<Item = &'a str>) -> Vec<regex::Regex> {
    names
        .filter(|name| !name.is_empty())
        .map(|name| {
            let escaped = regex::escape(name);
            regex::Regex::new(&format!(
                r"new\s+(?:[\w.:]+\.)?\b{escaped}(?:<[^>]+>)?\s*\(\s*async\b"
            ))
            .expect("constructor pattern")
        })
        .collect()
}

fn method_regexes<'a>(names: impl Iterator<Item = &'a str>) -> Vec<regex::Regex> {
    names
        .filter(|name| !name.is_empty())
        .map(|name| {
            let escaped = regex::escape(name);
            regex::Regex::new(&format!(r"(?:[\w.:]+\.)?\b{escaped}\s*\(\s*async\b"))
                .expect("method pattern")
        })
        .collect()
}

fn is_csharp_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("cs")
}

#[cfg(test)]
mod tests;
