use crate::config::v2::schema::{Project, TestPlanProjectDependency};

/// Project-relative path globs selected by a configured full-suite trigger.
///
/// This is shared by test planning and CI path-coverage analysis so both
/// consumers apply the same project-root and negation rules.
pub fn project_dependency_patterns(
    project_name: &str,
    project: &Project,
    trigger: &TestPlanProjectDependency,
) -> Vec<String> {
    match trigger {
        TestPlanProjectDependency::All(false) => Vec::new(),
        TestPlanProjectDependency::All(true) => {
            let root = project.root.as_deref().unwrap_or(project_name);
            if project.include.is_empty() {
                project_root_patterns(root)
            } else {
                project
                    .include
                    .iter()
                    .map(|pattern| project_relative_pattern(root, pattern))
                    .collect()
            }
        }
        TestPlanProjectDependency::Patterns(patterns) => {
            let root = project.root.as_deref().unwrap_or(project_name);
            patterns
                .iter()
                .map(|pattern| project_relative_pattern(root, pattern))
                .collect()
        }
        // Structured triggers still describe source-path coverage; their
        // runner-target restriction does not affect the selected paths.
        TestPlanProjectDependency::Targeted(targeted) => {
            let root = project.root.as_deref().unwrap_or(project_name);
            targeted
                .paths
                .iter()
                .map(|pattern| project_relative_pattern(root, pattern))
                .collect()
        }
    }
}

pub(crate) fn project_root_patterns(project_root: &str) -> Vec<String> {
    let root = normalize_project_glob_part(project_root);
    if root.is_empty() || root == "." {
        vec!["**".to_string()]
    } else {
        vec![format!("{root}/**")]
    }
}

pub(crate) fn project_relative_pattern(project_root: &str, pattern: &str) -> String {
    let (negated, pattern) = pattern
        .trim()
        .strip_prefix('!')
        .map_or((false, pattern.trim()), |pattern| (true, pattern));
    let root = normalize_project_glob_part(project_root);
    let pattern = normalize_project_glob_part(pattern);
    let joined = if root.is_empty() || root == "." || pattern.starts_with(&format!("{root}/")) {
        pattern
    } else {
        format!("{root}/{pattern}")
    };
    if negated {
        format!("!{joined}")
    } else {
        joined
    }
}

pub(crate) fn normalize_project_glob_part(raw: &str) -> String {
    let mut part = raw.trim().trim_matches('/').to_string();
    while let Some(rest) = part.strip_prefix("./") {
        part = rest.to_string();
    }
    part
}
