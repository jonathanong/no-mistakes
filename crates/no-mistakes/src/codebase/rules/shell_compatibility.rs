use super as rules;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use rules::{path_filter::GlobMatcher, RuleFinding};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod bash;
mod lex;
mod read_targets;
mod zsh;

pub use bash::RULE_ID as BASH_EMPTY_ARRAY_NOUNSET;
pub use zsh::RULE_ID as ZSH_READONLY_PARAMETER;

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(in crate::codebase::rules) struct Options {
    files: Vec<String>,
}

pub(crate) fn check_with_sources(
    id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let bash = id == BASH_EMPTY_ARRAY_NOUNSET;
    let mut findings = Vec::new();
    for rule in config.rule_applications(id) {
        let options: Options = rule.try_rule_options()?;
        let matcher = GlobMatcher::new(&options.files, &format!("{id} options.files"))?;
        if bash && matcher.is_empty() {
            continue;
        }
        let files = rules::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            all_files,
            sources,
            |path| {
                matcher.is_match(&relative_slash_path(root, path))
                    || (!bash && zsh::likely_shell_file(path))
            },
        )?;
        let batch: Vec<_> = files
            .par_iter()
            .flat_map_iter(|path| {
                let file = relative_slash_path(root, path);
                let selected = bash
                    || matcher.is_match(&file)
                    || path.extension().is_some_and(|ext| ext == "zsh");
                let src = match sources.read_source_path(path) {
                    Ok(src) => src,
                    Err(error) if selected => {
                        return vec![rules::source_access::read_failure(root, path, id, &error)];
                    }
                    Err(_) => return Vec::new(),
                };
                if !selected && !zsh::has_shebang(&src) {
                    return Vec::new();
                }
                let hits = if bash {
                    bash::scan(&src)
                } else {
                    zsh::scan(&src)
                };
                hits.into_iter()
                    .map(|(target, line)| RuleFinding {
                        causes: Vec::new(),
                        rule: id.into(),
                        file: file.clone(),
                        line,
                        message: rule
                            .message
                            .clone()
                            .unwrap_or_else(|| message(bash, target)),
                        import: None,
                        target: Some(target.into()),
                    })
                    .collect()
            })
            .collect();
        findings.extend(batch);
    }
    rules::sort_findings(&mut findings);
    Ok(findings)
}

fn message(bash: bool, target: &str) -> String {
    if bash {
        format!(
            "Array `{target}` can be empty. Bash before 4.4 treats an empty array as unbound under nounset. Wrap the quoted expansion in ${{{target}[@]+...}} to guard it and keep the original array selector."
        )
    } else {
        "Zsh reserves `status` as the read-only exit status. Rename the variable to `exit_status`."
            .into()
    }
}

pub(in crate::codebase::rules) fn validate_options(id: &str, options: &Options) -> Result<()> {
    GlobMatcher::new(&options.files, &format!("{id} options.files"))?;
    Ok(())
}
