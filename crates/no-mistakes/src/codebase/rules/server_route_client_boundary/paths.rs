use crate::config::v2::schema::{NoMistakesConfig, ProjectType, RuleDef};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

pub(super) fn route_globset_for_rule(config: &NoMistakesConfig, rule: &RuleDef) -> Option<GlobSet> {
    let mut globs = Vec::new();
    for (project_name, project) in &config.projects {
        if !rule.applies_to_repository()
            && !rule.projects.iter().any(|target| target == project_name)
        {
            continue;
        }
        if project.routes.is_empty()
            || project
                .type_
                .as_ref()
                .is_some_and(|type_| type_ != &ProjectType::Server)
        {
            continue;
        }
        let root = project.root.as_deref().unwrap_or(".");
        for route in &project.routes {
            globs.push(project_relative_glob(root, route));
        }
    }
    compile_globs(&globs)
}

pub(super) fn relative_path<'a>(root: &'a Path, path: &'a Path) -> &'a Path {
    path.strip_prefix(root).unwrap_or(path)
}

pub(super) struct ExcludeMatcher {
    globset: Option<GlobSet>,
    literal_components: HashSet<String>,
    literal_paths: Vec<PathBuf>,
}

impl ExcludeMatcher {
    pub(super) fn new(excludes: &[String]) -> Self {
        let mut builder = GlobSetBuilder::new();
        let mut has_globs = false;
        let mut literal_components = HashSet::new();
        let mut literal_paths = Vec::new();
        for exclude in excludes {
            let exclude = normalize_exclude(exclude);
            if has_glob_meta(&exclude) {
                if let Ok(glob) = GlobBuilder::new(&exclude).literal_separator(false).build() {
                    builder.add(glob);
                    has_globs = true;
                }
            } else if exclude.contains('/') {
                literal_paths.push(PathBuf::from(exclude));
            } else {
                literal_components.insert(exclude);
            }
        }
        let globset = has_globs.then(|| builder.build().ok()).flatten();
        Self {
            globset,
            literal_components,
            literal_paths,
        }
    }

    pub(super) fn is_match(&self, root: &Path, path: &Path) -> bool {
        let rel = relative_path(root, path);
        self.globset.as_ref().is_some_and(|set| set.is_match(rel))
            || self
                .literal_paths
                .iter()
                .any(|literal| rel == literal || rel.starts_with(literal))
            || rel.components().any(|component| match component {
                Component::Normal(name) => name
                    .to_str()
                    .is_some_and(|name| self.literal_components.contains(name)),
                _ => false,
            })
    }
}

fn project_relative_glob(root: &str, route: &str) -> String {
    let root = root.trim().trim_matches('/').trim_start_matches("./");
    let route = route
        .trim()
        .trim_start_matches('/')
        .trim_start_matches("./");
    if root.is_empty() || root == "." || route.starts_with(&format!("{root}/")) {
        route.to_string()
    } else {
        format!("{root}/{route}")
    }
}

fn normalize_exclude(exclude: &str) -> String {
    exclude
        .trim()
        .trim_start_matches('/')
        .trim_start_matches("./")
        .to_string()
}

fn compile_globs(globs: &[String]) -> Option<GlobSet> {
    if globs.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    let mut added = 0usize;
    for pattern in globs {
        if let Ok(glob) = GlobBuilder::new(pattern).literal_separator(false).build() {
            builder.add(glob);
            added += 1;
        }
    }
    (added > 0).then(|| builder.build().ok()).flatten()
}

fn has_glob_meta(pattern: &str) -> bool {
    pattern
        .chars()
        .any(|char| matches!(char, '*' | '?' | '[' | '{'))
}
