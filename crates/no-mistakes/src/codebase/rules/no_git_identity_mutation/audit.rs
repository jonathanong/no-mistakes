use super::super::allow_audit::{AllowAudit, CheckedScope};
use super::*;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use rayon::prelude::*;

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let files = super::super::selected_rule_files(root, config, rule, all_files)?;
        let audit = AllowAudit::new(
            root,
            config,
            rule,
            Some(sources),
            "conditionallyAllowedWorkflows",
            opts.conditionally_allowed_workflows.len(),
        );
        let exclusions = build_exclude_globset(&opts.exclude_paths);
        let conditional: Vec<_> = opts
            .conditionally_allowed_workflows
            .iter()
            .map(|pattern| Glob::new(pattern).ok().map(|glob| glob.compile_matcher()))
            .collect();
        let patterns = build_patterns();
        let checked = dashmap::DashSet::new();
        let raw: Vec<_> = files
            .par_iter()
            .flat_map(|path| {
                let Some(content) = super::super::read_source(sources, path) else {
                    return Vec::new();
                };
                let rel = relative_slash_path(root, path);
                if exclusions.is_match(&rel) {
                    return Vec::new();
                }
                let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                if !matches!(ext, "sh" | "bash" | "zsh" | "fish" | "yml" | "yaml")
                    && !has_shell_shebang(&content)
                {
                    return Vec::new();
                }
                if !opts.conditionally_allowed_workflows.is_empty() {
                    checked.insert(path.clone());
                }
                let raw = check_source(
                    path,
                    root,
                    &exclusions,
                    &GlobSet::empty(),
                    &patterns,
                    &content,
                );
                if !raw.is_empty()
                    && is_managed_runner_only(&content)
                    && audit.usage.suppress(conditional.iter().map(|matcher| {
                        matcher
                            .as_ref()
                            .is_some_and(|matcher| matcher.is_match(&rel))
                    }))
                {
                    return Vec::new();
                }
                raw
            })
            .collect();
        findings.extend(raw);
        let checked: Vec<_> = checked.iter().map(|path| path.key().clone()).collect();
        let scope = CheckedScope::new(root, Some(sources), &checked);
        findings.extend(audit.finish(
            |index| scope.glob(&opts.conditionally_allowed_workflows[index]),
            |index| opts.conditionally_allowed_workflows[index].clone(),
        ));
    }
    super::super::sort_findings(&mut findings);
    Ok(findings)
}
