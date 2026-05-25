use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;

pub(super) fn append_rule_globs(
    config: &NoMistakesConfig,
    rule: &RuleDef,
    includes: &mut Vec<String>,
    excludes: &mut Vec<String>,
) {
    includes.extend(rule.include.clone());
    excludes.extend(rule.exclude.clone());
    for project_name in &rule.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        let root = project.root.as_deref().unwrap_or(".").trim_matches('/');
        if root.is_empty() || root == "." {
            continue;
        }
        includes.extend(prefix_globs(root, &rule.include));
        excludes.extend(prefix_globs(root, &rule.exclude));
    }
}

pub(super) fn append_project_globs(
    config: &NoMistakesConfig,
    rule: &RuleDef,
    project_name: &str,
    includes: &mut Vec<String>,
    excludes: &mut Vec<String>,
) {
    let Some(project) = config.projects.get(project_name) else {
        return;
    };
    let root = project.root.as_deref().unwrap_or(".").trim_matches('/');
    let project_includes = if !rule.include.is_empty() {
        rule.include.clone()
    } else if project.include.is_empty() {
        vec!["**".to_string()]
    } else {
        project.include.clone()
    };
    append_prefixed(root, project_includes, includes);
    append_prefixed(root, project.exclude.clone(), excludes);
}

fn append_prefixed(root: &str, globs: Vec<String>, out: &mut Vec<String>) {
    for glob in globs {
        if root.is_empty() || root == "." {
            out.push(glob);
        } else {
            out.push(format!(
                "{}/{}",
                root.trim_start_matches("./"),
                glob.trim_start_matches("./")
            ));
        }
    }
}

fn prefix_globs(root: &str, globs: &[String]) -> Vec<String> {
    globs
        .iter()
        .map(|glob| {
            format!(
                "{}/{}",
                root.trim_start_matches("./"),
                glob.trim_start_matches("./")
            )
        })
        .collect()
}
