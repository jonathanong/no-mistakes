use super::analysis_policy::AnalysisDiagnostics;
use super::file_diagnostics::{self, InputUncertainty};
use super::masked_source::FileAllow;
use super::source_access;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod imports;
use imports::Import;

pub const RULE_ID: &str = "swift-no-unguarded-darwin-import";

const FIX: &str =
    "Wrap the Darwin import and the tests that use it in `#if canImport(Darwin)`.";
const UNPROVEN: &str = "Cannot verify that this compilation branch guards the Darwin import.";

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct Options {
    allow: Vec<String>,
    message: Option<String>,
}

pub(super) struct CompiledOptions {
    allow: FileAllow,
    message: String,
}

super::diagnostic_source_rule_entrypoints!();

pub(crate) fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let mut compiled = compile_options(rule.try_rule_options()?)?;
        compiled.allow.setup(root, config, rule, sources);
        let files = super::selected_rule_files_matching_with_sources(
            root, config, rule, all_files, sources, is_swift_file,
        )?;
        let mut findings: Vec<_> = files
            .par_iter()
            .flat_map_iter(|path| check_file(root, path, &compiled, sources))
            .collect();
        findings.extend(compiled.allow.finish(root, sources));
        diagnostics.append(file_diagnostics::application(
            root, sources, config, rule, findings, defer_suppression, InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(file_diagnostics::sorted(diagnostics))
}

pub(super) fn compile_options(options: Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        allow: FileAllow::new(&options.allow, &format!("{RULE_ID} options.allow"))?,
        message: match options.message.filter(|message| !message.is_empty()) {
            Some(message) => format!("{FIX} {message}"),
            None => FIX.to_string(),
        },
    })
}

fn is_swift_file(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("swift")
}

fn check_file(
    root: &Path,
    path: &Path,
    options: &CompiledOptions,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    let mut findings = match sources.read_source_path(path) {
        Ok(source) => {
            options.allow.checked(path);
            let code = super::lexical_mask::swift_declaration_mask(&source);
            imports::unproven_imports(&code)
                .map(|(line, kind)| finding(&file, line, kind, &options.message))
                .collect()
        }
        Err(error) => vec![source_access::read_failure(root, path, RULE_ID, &error)],
    };
    if options.allow.suppress(&file, !findings.is_empty()) {
        findings.clear();
    }
    findings
}

fn finding(file: &str, line: usize, kind: Import, fix: &str) -> RuleFinding {
    let (message, target) = match kind {
        Import::Unguarded => (fix.to_owned(), None),
        Import::Unproven => (format!("{UNPROVEN} {fix}"), Some("unanalyzable".to_owned())),
    };
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: file.to_owned(),
        line,
        message,
        import: None,
        target,
        causes: Vec::new(),
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
