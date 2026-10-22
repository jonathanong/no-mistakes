use super::super::RuleFinding;
use super::config::{Options, Target, UnknownArguments, UnknownCalls};
use super::RULE_ID;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite};
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

pub(super) fn finding_for_source_call(
    root: &Path,
    application: &RuleDef,
    label: &str,
    options: &Options,
    file: &Path,
    call: &crate::codebase::dependencies::extract::FunctionCall,
    arguments: Option<&crate::codebase::ts_source::module_facts::arguments::ArgumentValues>,
) -> Option<RuleFinding> {
    let target = options.targets.iter().find_map(|target| {
        let Target::Exact(exact) = target else {
            return None;
        };
        if exact.exact != call.callee {
            return None;
        }
        let description = describe_target(target);
        let Some(conditions) = exact.when_arguments.as_ref() else {
            return Some(description);
        };
        match super::arguments::matches(conditions, arguments) {
            super::arguments::Match::Yes => Some(description),
            super::arguments::Match::Unknown
                if options.unknown_arguments == UnknownArguments::Finding =>
            {
                Some(format!("unknown arguments for {description}"))
            }
            _ => None,
        }
    })?;
    let message = application.message.as_deref().unwrap_or("forbidden call");
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, file),
        line: call.line as usize,
        message: format!("{message} ({label}): {target}"),
        import: Some(call.callee.clone()),
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
    (site.has_unknown_target() && unknown_calls == &UnknownCalls::Finding)
        .then(|| "unknown call".to_string())
}
fn target_matches(root: &Path, selector: &Target, site: &ResolvedCallSite) -> bool {
    if let Target::Exact(selector) = selector {
        return site.source_callee == selector.exact;
    }
    if let Target::Terminal(selector) = selector {
        if site.has_unknown_target()
            && site
                .source_callee
                .rsplit_once('.')
                .is_some_and(|(_, terminal)| terminal == selector.terminal)
        {
            return true;
        }
    }
    site.targets().any(|candidate| match selector {
        Target::Global(selector) => matches!(candidate.target,
            KnownCallTargetRef::Global { name } if name == selector.global),
        Target::Exact(_) => false,
        Target::Terminal(selector) => terminal_name(candidate.target) == selector.terminal,
        Target::ModuleExport(selector) => matches!(candidate.target,
            KnownCallTargetRef::ModuleExport { specifier, export_path, .. }
                if specifier == selector.module_export.module && export_path == selector.module_export.export),
        Target::Function(selector) => {
            let expected_file = crate::codebase::ts_resolver::normalize_path(&root.join(&selector.function.file));
            match candidate.target {
                KnownCallTargetRef::RepositoryFunction { file, scope }
                | KnownCallTargetRef::ModuleExport { repository_target: Some((file, scope)), .. } =>
                    file == expected_file.as_path() && scope == selector.function.symbol,
                _ => false,
            }
        }
    })
}
fn terminal_name(target: KnownCallTargetRef<'_>) -> &str {
    match target {
        KnownCallTargetRef::Global { name } => name.rsplit('.').next().unwrap_or(name),
        KnownCallTargetRef::ModuleExport { export_path, .. } => {
            export_path.rsplit('.').next().unwrap_or(export_path)
        }
        KnownCallTargetRef::RepositoryFunction { scope, .. } => {
            scope.rsplit('/').next().unwrap_or(scope)
        }
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
