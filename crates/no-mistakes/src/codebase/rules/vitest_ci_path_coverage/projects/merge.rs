use crate::integration_tests::types::ConfigProject;

#[cfg(test)]
mod tests;

pub(super) fn merge_explicit_project(projects: &mut Vec<ConfigProject>, project: ConfigProject) {
    let Some(existing) = projects
        .iter_mut()
        .find(|existing| existing.policy_name.as_deref() == project.policy_name.as_deref())
    else {
        projects.push(project);
        return;
    };

    if project.config.is_some() {
        existing.config = project.config;
    }
    if project.runner_project_arg.is_some() {
        existing.runner_project_arg = project.runner_project_arg;
    }
    if project.scope.is_some() {
        existing.scope = project.scope;
    }
    if !project.include.is_empty() {
        existing.include = project.include;
    }
    if !project.exclude.is_empty() {
        existing.exclude = project.exclude;
    }
}
