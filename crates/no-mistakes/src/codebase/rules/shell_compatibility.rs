use super as rules;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use rules::{analysis_policy::AnalysisDiagnostics, path_filter::GlobMatcher, RuleFinding};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod bash;
mod format;
mod lex;
mod read_targets;
mod zsh;

pub use bash::RULE_ID as BASH_EMPTY_ARRAY_NOUNSET;
pub(in crate::codebase::rules) use format::validate_options;
pub use zsh::RULE_ID as ZSH_READONLY_PARAMETER;

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(in crate::codebase::rules) struct Options {
    files: Vec<String>,
}

pub fn bash_diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    diagnostics_for_rule(
        BASH_EMPTY_ARRAY_NOUNSET,
        root,
        config,
        all_files,
        sources,
        defer_suppression,
    )
}

pub(crate) fn check_with_sources(
    id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    diagnostics_for_rule(id, root, config, all_files, sources, true).map(|d| d.findings)
}

fn diagnostics_for_rule(
    id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let bash = id == BASH_EMPTY_ARRAY_NOUNSET;
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(id) {
        let options: Options = rule.try_rule_options()?;
        let matcher = GlobMatcher::new(&options.files, &format!("{id} options.files"))?;
        if bash && matcher.is_empty() {
            continue;
        }
        let unanalyzable = if bash {
            resolve_unanalyzable(config, rule)?
        } else {
            Unanalyzable::Advise
        };
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
        let ctx = ScanCtx {
            id,
            bash,
            root,
            matcher: &matcher,
            sources,
            rule,
            unanalyzable,
        };
        let chunks: Vec<_> = files.par_iter().map(|path| scan_file(&ctx, path)).collect();
        for chunk in chunks {
            output.append(chunk);
        }
    }
    if !defer_suppression {
        rules::suppress_rule_findings_with_sources(root, &mut output.findings, sources);
        rules::suppress_rule_findings_with_sources(root, &mut output.advisories, sources);
    }
    rules::sort_findings(&mut output.findings);
    output.findings.dedup();
    rules::sort_findings(&mut output.advisories);
    output.advisories.dedup();
    Ok(output)
}

struct ScanCtx<'a> {
    id: &'a str,
    bash: bool,
    root: &'a Path,
    matcher: &'a GlobMatcher,
    sources: &'a SourceStore,
    rule: &'a crate::config::v2::schema::RuleDef,
    unanalyzable: Unanalyzable,
}

fn scan_file(ctx: &ScanCtx<'_>, path: &Path) -> AnalysisDiagnostics {
    let mut diag = AnalysisDiagnostics::default();
    let file = relative_slash_path(ctx.root, path);
    let selected =
        ctx.bash || ctx.matcher.is_match(&file) || path.extension().is_some_and(|ext| ext == "zsh");
    let src = match ctx.sources.read_source_path(path) {
        Ok(src) => src,
        Err(error) if selected => {
            diag.findings.push(rules::source_access::read_failure(
                ctx.root, path, ctx.id, &error,
            ));
            return diag;
        }
        Err(_) => return diag,
    };
    if !selected && !zsh::has_shebang(&src) {
        return diag;
    }
    if ctx.bash {
        let result = bash::analyze(&src);
        for (target, line) in result.findings {
            let msg = ctx
                .rule
                .message
                .clone()
                .unwrap_or_else(|| format::message(true, target));
            diag.findings
                .push(format::finding(ctx.id, &file, line, target, msg));
        }
        for (line, detail) in result.unanalyzable {
            let msg = format!("Cannot prove bash array safety: {detail}");
            diag.unanalyzable(
                ctx.unanalyzable,
                format::finding(ctx.id, &file, line, "unanalyzable", msg),
            );
        }
    } else {
        for (target, line) in zsh::scan(&src) {
            let msg = ctx
                .rule
                .message
                .clone()
                .unwrap_or_else(|| format::message(false, target));
            diag.findings
                .push(format::finding(ctx.id, &file, line, target, msg));
        }
    }
    diag
}

fn resolve_unanalyzable(
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<Unanalyzable> {
    let explicit = rule.options.get("unanalyzable").is_some() || config.unanalyzable.is_some();
    if explicit {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
}
