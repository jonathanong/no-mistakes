use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "no-test-git-sha";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    /// Regexes whose matching literal is an intentional generated/ref-shape assertion.
    pub(crate) allowed_contexts: Vec<String>,
}
struct CompiledOptions {
    contexts: Vec<Regex>,
    sha: Regex,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    check_with_files_and_sources(root, config, files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let results: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let options = compile_options(rule.try_rule_options()?)?;
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let candidates: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let candidates =
                super::path_filter::filter_rule_files(root, config, rule, &candidates)?;
            Ok(candidates
                .par_iter()
                .flat_map(|path| check_file(root, path, &options, sources))
                .collect())
        })
        .collect();
    let mut findings: Vec<RuleFinding> = results?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(options: Options) -> Result<CompiledOptions> {
    let contexts = options
        .allowed_contexts
        .iter()
        .map(|pattern| {
            Regex::new(pattern)
                .with_context(|| format!("{RULE_ID} options.allowedContexts regex `{pattern}`"))
        })
        .collect::<Result<_>>()?;
    Ok(CompiledOptions {
        contexts,
        sha: Regex::new(r"(?i)[0-9a-f]{40}").expect("SHA regex"),
    })
}

fn check_file(
    root: &Path,
    path: &Path,
    options: &CompiledOptions,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let Some(content) = super::read_source(sources, path) else {
        return Vec::new();
    };
    let file = relative_slash_path(root, path);
    content.lines().enumerate().flat_map(|(index, line_text)| {
        let line = index + 1;
        let allowed = options.contexts.iter().any(|context| context.is_match(line_text));
        options
            .sha
            .find_iter(line_text)
            .filter(|matched| {
                let bytes = line_text.as_bytes();
                (matched.start() == 0 || !bytes[matched.start() - 1].is_ascii_hexdigit())
                    && (matched.end() == bytes.len()
                        || !bytes[matched.end()].is_ascii_hexdigit())
            })
            .filter(|_| !allowed)
            .map(|_| RuleFinding {
                rule: RULE_ID.to_string(),
                file: file.clone(),
                line,
                message: format!(
                    "{file}:{line}: test source embeds a full Git SHA; assert a generated or SHA-shaped ref instead"
                ),
                import: None,
                target: None,
            })
            .collect::<Vec<_>>()
    }).collect()
}

#[cfg(test)]
mod tests;
