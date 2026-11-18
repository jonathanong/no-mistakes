use crate::config::v2::schema::ProjectWaitScopes;
use globset::{GlobSet, GlobSetBuilder};
use std::path::Path;

pub(crate) struct WaitScopes<'a> {
    projects: &'a [ProjectWaitScopes],
    patterns: Vec<Option<Result<GlobSet, globset::Error>>>,
}

impl<'a> WaitScopes<'a> {
    pub(crate) fn new(projects: &'a [ProjectWaitScopes]) -> Self {
        Self {
            projects,
            patterns: projects
                .iter()
                .map(|project| compile(&project.scopes))
                .collect(),
        }
    }

    pub(crate) fn matches(&self, path: &Path, root: &Path) -> bool {
        let (relative, project) = select(path, root, self.projects);
        project.is_none_or(|index| matches(self.patterns[index].as_ref(), &relative))
    }
}

pub(crate) fn waits_in_scope(path: &Path, root: &Path, projects: &[ProjectWaitScopes]) -> bool {
    let (relative, project) = select(path, root, projects);
    project.is_none_or(|index| matches(compile(&projects[index].scopes).as_ref(), &relative))
}

fn matches(pattern: Option<&Result<GlobSet, globset::Error>>, path: &str) -> bool {
    pattern.is_none_or(|matcher| {
        matcher
            .as_ref()
            .is_ok_and(|set| set.is_match(Path::new(path)))
    })
}

fn select(path: &Path, root: &Path, projects: &[ProjectWaitScopes]) -> (String, Option<usize>) {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let relative = path
        .strip_prefix(&root)
        .unwrap_or(&path)
        .to_string_lossy()
        .replace('\\', "/");
    let project = projects
        .iter()
        .enumerate()
        .filter(|(_, project)| root_contains(&relative, &project.root))
        .max_by_key(|(_, project)| project.root.len())
        .map(|(index, _)| index);
    (relative, project)
}

fn root_contains(path: &str, root: &str) -> bool {
    let root = root.trim().trim_matches('/');
    root.is_empty() || root == "." || path == root || path.starts_with(&format!("{root}/"))
}

fn compile(scopes: &[String]) -> Option<Result<GlobSet, globset::Error>> {
    let mut builder = GlobSetBuilder::new();
    let mut added = false;
    for scope in scopes {
        for pattern in [scope.clone(), format!("**/{scope}")] {
            if let Ok(glob) = globset::GlobBuilder::new(&pattern)
                .literal_separator(false)
                .build()
            {
                builder.add(glob);
                added = true;
            }
        }
    }
    added.then(|| builder.build())
}
