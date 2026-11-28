use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use super::file_diagnostics::{self, InputUncertainty};
use super::masked_source::FileAllow;
use super::source_access;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use no_mistakes_rule_kernel::finding::RuleFinding;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod conditions;
mod imports;
mod packages;
use imports::Import;
use packages::{FileScope, PackageIndex};

pub const RULE_ID: &str = "swift-no-unguarded-darwin-import";

const DEFAULT_MODULES: [&str; 1] = ["Darwin"];

#[derive(Deserialize)]
#[serde(default)]
pub(super) struct Options {
    allow: Vec<String>,
    message: Option<String>,
    modules: Vec<ModuleName>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            allow: Vec::new(),
            message: None,
            modules: Vec::from(DEFAULT_MODULES.map(|n| ModuleName(n.to_owned()))),
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct ModuleName(String);

impl TryFrom<String> for ModuleName {
    type Error = &'static str;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if !imports::valid_module_name(&name) {
            return Err("expected a bare Swift module identifier");
        }
        Ok(Self(name))
    }
}

pub(super) struct CompiledOptions {
    allow: FileAllow,
    message: Option<String>,
    modules: FxHashSet<String>,
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
    let package_index = PackageIndex::from_files(all_files, sources);
    for rule in config.rule_applications(RULE_ID) {
        let mut compiled = compile_options(rule.try_rule_options()?)?;
        compiled.allow.setup(root, config, rule, sources);
        let files = super::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            all_files,
            sources,
            is_swift_file,
        )?;
        let mut findings: Vec<_> = files
            .par_iter()
            .flat_map_iter(|path| check_file(root, path, &compiled, &package_index, sources))
            .collect();
        findings.extend(compiled.allow.finish(root, sources));
        diagnostics.append(file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(file_diagnostics::sorted(diagnostics))
}

pub(super) fn compile_options(options: Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        allow: FileAllow::new(&options.allow, &format!("{RULE_ID} options.allow"))?,
        message: options.message.filter(|message| !message.is_empty()),
        modules: options.modules.into_iter().map(|name| name.0).collect(),
    })
}

fn is_swift_file(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("swift")
}

fn check_file(
    root: &Path,
    path: &Path,
    options: &CompiledOptions,
    package_index: &PackageIndex,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let scope = package_index.classify(path);
    if matches!(scope, FileScope::Skip) {
        return Vec::new();
    }
    let file = relative_slash_path(root, path);
    let mut findings = match sources.read_source_path(path) {
        Ok(source) => {
            options.allow.checked(path);
            let code = super::lexical_mask::swift_declaration_mask(&source);
            imports::unproven_imports(&code, &options.modules)
                .map(|(line, module, kind)| finding(&file, line, module, kind, &scope, options))
                .collect()
        }
        Err(error) => vec![source_access::read_failure(root, path, RULE_ID, &error)],
    };
    if options.allow.suppress(&file, !findings.is_empty()) {
        findings.clear();
    }
    findings
}

fn finding(
    file: &str,
    line: usize,
    module: &str,
    kind: Import,
    scope: &FileScope,
    options: &CompiledOptions,
) -> RuleFinding {
    let fix =
        format!("Wrap the {module} import and the tests that use it in `#if canImport({module})`.");
    let (mut message, target) = match (scope, kind) {
        (FileScope::Uncertain(reason), _) => (
            format!("Cannot verify whether this file belongs to a portable SwiftPM test target ({reason}). {fix}"),
            Some("unanalyzable".to_owned()),
        ),
        (_, Import::Unguarded) => (fix, None),
        (_, Import::Unproven) => (
            format!("Cannot verify that this compilation branch guards the {module} import. {fix}"),
            Some("unanalyzable".to_owned()),
        ),
    };
    if let Some(hint) = options.message.as_deref() {
        message.push(' ');
        message.push_str(hint);
    }
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
