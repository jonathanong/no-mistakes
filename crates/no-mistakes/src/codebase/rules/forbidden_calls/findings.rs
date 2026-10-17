use super::super::RuleFinding;
use super::config::{Options, Target, UnknownArguments, UnknownCalls};
use super::RULE_ID;
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::RuleDef;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn finding_for_site_with_arguments(
    root: &Path,
    application: &RuleDef,
    application_label: &str,
    options: &Options,
    site: &ResolvedCallSite,
    arguments: Option<&crate::codebase::ts_source::module_facts::arguments::ArgumentValues>,
) -> Option<RuleFinding> {
    let target = target_label(
        root,
        site,
        &options.targets,
        &options.unknown_calls,
        &options.unknown_arguments,
        arguments,
    )?;
    let message = application.message.as_deref().unwrap_or("forbidden call");
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, &site.file),
        line: site.line as usize,
        message: format!("{message} ({application_label}): {target}"),
        import: Some(site.source_callee.clone()),
        target: Some(target),
    })
}
fn target_label(
    root: &Path,
    site: &ResolvedCallSite,
    targets: &[Target],
    unknown_calls: &UnknownCalls,
    unknown_arguments: &UnknownArguments,
    arguments: Option<&crate::codebase::ts_source::module_facts::arguments::ArgumentValues>,
) -> Option<String> {
    for target in targets {
        if !target_matches(root, target, site) {
            continue;
        }
        let Some(conditions) = target.arguments() else {
            return Some(describe_target(target));
        };
        match super::arguments::matches(conditions, arguments) {
            super::arguments::Match::Yes => return Some(describe_target(target)),
            super::arguments::Match::Unknown if unknown_arguments == &UnknownArguments::Finding => {
                return Some(format!("unknown arguments for {}", describe_target(target)));
            }
            _ => {}
        }
    }
    (matches!(site.target, ResolvedCallTarget::Unknown) && unknown_calls == &UnknownCalls::Finding)
        .then(|| "unknown call".to_string())
}
fn target_matches(root: &Path, selector: &Target, site: &ResolvedCallSite) -> bool {
    match selector {
        Target::Global(selector) => {
            let global = &selector.global;
            matches!(&site.target, ResolvedCallTarget::Global { name } if name == global)
        }
        Target::Exact(selector) => site.source_callee == selector.exact,
        Target::Terminal(selector) => {
            terminal_name(site).is_some_and(|name| name == selector.terminal)
        }
        Target::ModuleExport(selector) => {
            let module_export = &selector.module_export;
            matches!(&site.target, ResolvedCallTarget::ModuleExport { specifier, export_path, .. } if specifier == &module_export.module && export_path == &module_export.export)
        }
        Target::Function(selector) => {
            let expected_file =
                crate::codebase::ts_resolver::normalize_path(&root.join(&selector.function.file));
            let expected_symbol = &selector.function.symbol;
            match &site.target {
                ResolvedCallTarget::RepositoryFunction { file, scope } => {
                    file == &expected_file && scope == expected_symbol
                }
                ResolvedCallTarget::ModuleExport {
                    repository_target: Some((file, scope)),
                    ..
                } => file == &expected_file && scope == expected_symbol,
                _ => false,
            }
        }
    }
}
fn terminal_name(site: &ResolvedCallSite) -> Option<&str> {
    match &site.target {
        ResolvedCallTarget::Global { name } => name.rsplit('.').next(),
        ResolvedCallTarget::ModuleExport { export_path, .. } => export_path.rsplit('.').next(),
        ResolvedCallTarget::RepositoryFunction { scope, .. } => scope.rsplit('/').next(),
        // Typed parameters and other unresolved member calls keep source spelling.
        ResolvedCallTarget::Unknown => site
            .source_callee
            .rsplit_once('.')
            .map(|(_, terminal)| terminal),
    }
}
fn describe_target(selector: &Target) -> String {
    match selector {
        Target::Global(selector) => format!("global `{}`", selector.global),
        Target::Exact(selector) => format!("exact `{}`", selector.exact),
        Target::Terminal(selector) => format!("terminal `{}`", selector.terminal),
        Target::ModuleExport(selector) => format!(
            "module export `{}#{}`",
            selector.module_export.module, selector.module_export.export
        ),
        Target::Function(selector) => format!(
            "repository function `{}#{}`",
            selector.function.file, selector.function.symbol
        ),
    }
}

pub(super) fn finish(findings: Vec<(u32, RuleFinding)>) -> Vec<RuleFinding> {
    let mut totals = BTreeMap::new();
    for (_, finding) in &findings {
        *totals.entry(finding.clone()).or_insert(0usize) += 1;
    }
    let mut seen = BTreeMap::new();
    let mut findings = findings
        .into_iter()
        .map(|(_, mut finding)| {
            if totals.get(&finding).copied().unwrap_or_default() > 1 {
                let occurrence = seen.entry(finding.clone()).or_insert(0usize);
                *occurrence += 1;
                finding.message = format!("{} (occurrence {})", finding.message, occurrence);
            }
            finding
        })
        .collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        (&left.file, left.line, &left.message, &left.target).cmp(&(
            &right.file,
            right.line,
            &right.message,
            &right.target,
        ))
    });
    findings
}
