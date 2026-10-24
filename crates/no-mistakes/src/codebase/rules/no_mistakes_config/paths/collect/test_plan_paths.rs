use super::{frameworks, path_kind, push, Field, Ref};
use crate::codebase::workflow_topology::posix_path::normalize;
use crate::config::v2::{schema::TestPlanProjectDependency, NoMistakesConfig};

pub(super) fn collect(config: &NoMistakesConfig, refs: &mut Vec<Ref>, origins: bool) {
    for (framework, plan) in frameworks(config) {
        let field = Field::new("testPlan", origins)
            .key(framework)
            .key("fullSuiteTriggers");
        for (project_name, dependency) in &plan.full_suite_triggers.projects {
            let root = config
                .projects
                .get(project_name)
                .and_then(|project| project.root.as_deref())
                .unwrap_or("");
            let project = field.key("projects").key(project_name);
            let (paths, authored) = match dependency {
                TestPlanProjectDependency::Patterns(paths) => (paths, project),
                TestPlanProjectDependency::Targeted(targeted) => {
                    (&targeted.paths, project.authored_key("paths"))
                }
                TestPlanProjectDependency::All(_) => continue,
            };
            for (index, path) in paths.iter().enumerate() {
                if let Some(path) = project_relative(root, path) {
                    push(refs, authored.index(index), path_kind(&path), &path);
                }
            }
        }
        for (index, trigger) in plan.full_suite_triggers.triggers.iter().enumerate() {
            for (path_index, path) in trigger.paths.iter().enumerate() {
                if !path.starts_with('!') {
                    push(
                        refs,
                        field
                            .key("triggers")
                            .index(index)
                            .key("paths")
                            .index(path_index),
                        path_kind(path),
                        path,
                    );
                }
            }
        }
    }
}

fn project_relative(root: &str, value: &str) -> Option<String> {
    if value.trim().is_empty() || value.starts_with('!') {
        return None;
    }
    let root = root.trim().trim_matches('/');
    let value = value.trim().trim_start_matches("./");
    Some(normalize(&if root.is_empty() || root == "." {
        value.to_string()
    } else {
        format!("{root}/{value}")
    }))
}
