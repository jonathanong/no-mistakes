use super::{ast, runtime_deps, RULE_ID};
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_resolver::ImportResolver;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct DynamicImportKey {
    pub(super) file: PathBuf,
    pub(super) line: usize,
    pub(super) specifier: Option<String>,
}

pub(super) struct DynamicImportOutcome {
    pub(super) key: DynamicImportKey,
    pub(super) covered: bool,
    pub(super) findings: Vec<RuleFinding>,
}

pub(super) struct DynamicCheckContext<'a> {
    pub(super) root: &'a Path,
    pub(super) file: &'a Path,
    pub(super) resolver: &'a ImportResolver<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) file_universe: Option<&'a HashSet<PathBuf>>,
    pub(super) mocks: &'a HashSet<PathBuf>,
    pub(super) dependency_cache: &'a DashMap<PathBuf, Arc<Vec<PathBuf>>>,
    pub(super) findings: &'a mut Vec<RuleFinding>,
}

pub(super) fn check_dynamic_import(ctx: &mut DynamicCheckContext<'_>, import: ast::DynamicImport) {
    let outcome = evaluate_dynamic_import(ctx, import);
    ctx.findings.extend(outcome.findings);
}

pub(super) fn evaluate_dynamic_import(
    ctx: &DynamicCheckContext<'_>,
    import: ast::DynamicImport,
) -> DynamicImportOutcome {
    let key = DynamicImportKey {
        file: ctx.file.to_path_buf(),
        line: import.line,
        specifier: import.specifier.clone(),
    };
    let Some(specifier) = import.specifier else {
        return DynamicImportOutcome {
            key,
            covered: false,
            findings: vec![build_finding(ctx.root, ctx.file, import.line, None, None)],
        };
    };
    let Some(target) = ctx.resolver.resolve(&specifier, ctx.file) else {
        if ctx.mocks.contains(&PathBuf::from(&specifier)) {
            return DynamicImportOutcome {
                key,
                covered: true,
                findings: Vec::new(),
            };
        }
        return DynamicImportOutcome {
            key,
            covered: false,
            findings: vec![build_finding(
                ctx.root,
                ctx.file,
                import.line,
                Some(specifier),
                None,
            )],
        };
    };
    if ctx.mocks.contains(&target) {
        return DynamicImportOutcome {
            key,
            covered: true,
            findings: Vec::new(),
        };
    }
    let deps = ctx
        .dependency_cache
        .entry(target.clone())
        .or_insert_with(|| Arc::new(runtime_deps(ctx.graph, target.clone(), ctx.file_universe)))
        .clone();
    let mut findings = Vec::new();
    for dependency in std::iter::once(&target).chain(deps.iter()) {
        if !ctx.mocks.contains(dependency) {
            findings.push(build_finding(
                ctx.root,
                ctx.file,
                import.line,
                Some(specifier.clone()),
                Some(dependency.clone()),
            ));
        }
    }
    DynamicImportOutcome {
        key,
        covered: false,
        findings,
    }
}

fn build_finding(
    root: &Path,
    file: &Path,
    line: usize,
    specifier: Option<String>,
    target: Option<PathBuf>,
) -> RuleFinding {
    let rel_file = crate::codebase::ts_source::relative_slash_path(root, file);
    let rel_target = target
        .as_ref()
        .map(|path| crate::codebase::ts_source::relative_slash_path(root, path));
    let label = rel_target
        .as_deref()
        .or(specifier.as_deref())
        .unwrap_or("dynamic import")
        .to_string();
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel_file,
        line,
        import: specifier,
        target: rel_target,
        message: format!("dynamic import dependency `{label}` must be mocked"),
    }
}
