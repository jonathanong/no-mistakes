use super::masked_source::FileAllow;
use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

mod scan;

pub const RULE_ID: &str = "swift-no-raw-print";

pub(super) const DEFAULT_MESSAGE: &str = "use a logger instead of raw print()";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) allow: Vec<String>,
    pub(crate) message: Option<String>,
}

pub(super) struct CompiledOptions {
    pub(super) print: regex::Regex,
    pub(super) allow: FileAllow,
    pub(super) message: String,
}

super::masked_source_rule_entrypoints!(compile_options, is_swift_file, scan::check_file);

fn compile_options(opts: Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        print: regex::Regex::new(r"(?:^|[^\w.])(?:Swift\.)?print\s*\(").expect("print pattern"),
        allow: FileAllow::new(&opts.allow, &format!("{RULE_ID} allow"))?,
        message: match opts.message.filter(|hint| !hint.is_empty()) {
            Some(hint) => format!("{DEFAULT_MESSAGE} {hint}"),
            None => DEFAULT_MESSAGE.to_string(),
        },
    })
}

fn is_swift_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("swift")
}

#[cfg(test)]
mod tests;
