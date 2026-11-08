use super::{NoMistakesConfig, Options, Path};
use crate::codebase::rules::{
    allow_audit::{is_literal_file, missing_file_findings},
    RuleFinding,
};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::schema::RuleDef;

pub(super) fn missing_findings(
    root: &Path,
    project_root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    sources: &SourceStore,
    opts: &Options,
    selection: (&super::GlobMatcher, &super::GlobMatcher),
) -> Vec<RuleFinding> {
    let (include, exclude) = selection;
    let mut findings = Vec::new();
    for (option, alias, entries, component) in [
        ("allowFiles", "allow_files", &opts.allow_files, false),
        (
            "allowComponents",
            "allow_components",
            &opts.allow_components,
            true,
        ),
    ] {
        let option = if rule.options.get(option).is_some() {
            option
        } else {
            alias
        };
        findings.extend(missing_file_findings(
            root,
            config,
            rule,
            sources,
            option,
            entries.keys().map(|key| {
                let file = if component {
                    key.rsplit_once('#').map_or(key.as_str(), |(file, _)| file)
                } else {
                    key.as_str()
                };
                let path = crate::codebase::ts_resolver::normalize_path(&project_root.join(file));
                let relative = relative_slash_path(project_root, &path);
                (
                    if is_literal_file(file)
                        && (include.is_empty() || include.is_match(&relative))
                        && !exclude.is_match(&relative)
                    {
                        relative_slash_path(root, &path)
                    } else {
                        String::new()
                    },
                    key.clone(),
                )
            }),
        ));
    }
    findings
}
