use super::policy::ExcludeEntry;
use super::{rel, RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn included_exclude(
    root: &Path,
    workspace: &Path,
    sources: &SourceStore,
    fallback: Vec<ExcludeEntry>,
) -> (Vec<ExcludeEntry>, Vec<RuleFinding>) {
    let Some(workspace_root) = workspace.parent() else {
        return (fallback, Vec::new());
    };
    let Ok(source) = sources.read_path(workspace) else {
        return (fallback, Vec::new());
    };
    let Ok(resolved) =
        crate::codebase::workspaces::resolve_pnpm_include(workspace_root, &source, |path| {
            sources.read_path(path).ok()
        })
    else {
        return (fallback, Vec::new());
    };
    let file = rel(root, workspace);
    let mut notes = Vec::new();
    if resolved
        .duplicates
        .iter()
        .any(|setting| setting == "minimumReleaseAgeExclude")
    {
        notes.push(finding(
            file.clone(),
            format!(
                "{file}: setting \"minimumReleaseAgeExclude\" is claimed by more than one pnpm source"
            ),
        ));
    }
    for issue in resolved
        .load_errors
        .iter()
        .filter(|issue| issue.affects("minimumReleaseAgeExclude"))
    {
        let mut note = finding(file.clone(), format!("{file}: {}", issue.message));
        note.target = Some("unanalyzable".to_owned());
        notes.push(note);
    }
    let exclude = match resolved.release_age_exclude {
        crate::codebase::workspaces::ReleaseAgeExclude::FromRoot => fallback,
        crate::codebase::workspaces::ReleaseAgeExclude::Included(values) => values
            .iter()
            .map(|entry| match entry.as_str() {
                Some(name) => ExcludeEntry::Name(name.to_string()),
                None => ExcludeEntry::Other,
            })
            .collect(),
        crate::codebase::workspaces::ReleaseAgeExclude::Conflict => vec![ExcludeEntry::Conflict],
    };
    (exclude, notes)
}

fn finding(file: String, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file,
        line: 1,
        message,
        import: None,
        target: Some("minimumReleaseAgeExclude".to_string()),
    }
}
