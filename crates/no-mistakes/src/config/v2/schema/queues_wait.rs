use super::super::NoMistakesConfig;

/// Wait-check globs for one project root. An empty `scopes` list checks every
/// file under `root`. A file outside every project stays in scope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectWaitScopes {
    pub root: String,
    pub scopes: Vec<String>,
}

pub(super) fn attach(scopes: &mut Vec<ProjectWaitScopes>, config: &NoMistakesConfig) {
    for project in config.projects.values() {
        let mut project_scopes = project
            .queues
            .wait_scopes
            .iter()
            .map(|scope| prefix_scope(project.root.as_deref(), scope))
            .collect::<Vec<_>>();
        project_scopes.sort();
        project_scopes.dedup();
        scopes.push(ProjectWaitScopes {
            root: project_root(project.root.as_deref()),
            scopes: project_scopes,
        });
    }
    dedup(scopes);
}

pub(super) fn merge(left: &mut Vec<ProjectWaitScopes>, right: &[ProjectWaitScopes]) {
    for project in right {
        if let Some(existing) = left
            .iter_mut()
            .find(|candidate| candidate.root == project.root)
        {
            union_scopes(existing, &project.scopes);
        } else {
            left.push(project.clone());
        }
    }
}

pub(super) fn dedup(scopes: &mut Vec<ProjectWaitScopes>) {
    scopes.sort_by(|left, right| left.root.cmp(&right.root));
    let mut merged: Vec<ProjectWaitScopes> = Vec::new();
    for project in scopes.drain(..) {
        if let Some(existing) = merged
            .iter_mut()
            .find(|candidate| candidate.root == project.root)
        {
            union_scopes(existing, &project.scopes);
        } else {
            merged.push(project);
        }
    }
    *scopes = merged;
}

/// An empty list checks every file of the project, so it absorbs narrower globs.
fn union_scopes(existing: &mut ProjectWaitScopes, scopes: &[String]) {
    if existing.scopes.is_empty() || scopes.is_empty() {
        existing.scopes.clear();
        return;
    }
    existing.scopes.extend(scopes.iter().cloned());
    existing.scopes.sort();
    existing.scopes.dedup();
}

fn project_root(root: Option<&str>) -> String {
    root.map(str::trim)
        .map(|root| root.trim_start_matches("./").trim_end_matches('/'))
        .filter(|root| !root.is_empty() && *root != ".")
        .unwrap_or("")
        .to_string()
}

fn prefix_scope(root: Option<&str>, glob: &str) -> String {
    let glob = glob.trim_start_matches("./");
    let root = project_root(root);
    if root.is_empty() || glob == root || glob.starts_with(&format!("{root}/")) {
        glob.to_string()
    } else {
        format!("{root}/{glob}")
    }
}
