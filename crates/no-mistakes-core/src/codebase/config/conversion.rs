use super::{Config, FilesystemConfig, ProjectConfig, RuleConfig};
use crate::config::v2::schema::NoMistakesConfig;
use std::collections::{HashMap, HashSet};

pub(super) fn config_from_v2(v2: NoMistakesConfig) -> Config {
    let mut projects: HashMap<String, ProjectConfig> = v2
        .projects
        .into_iter()
        .map(|(name, project)| {
            (
                name,
                ProjectConfig {
                    type_: project.type_,
                    root: project.root,
                    include: project.include,
                    rules: Vec::new(),
                },
            )
        })
        .collect();
    let mut rules = HashMap::new();
    let mut repository_rules = HashSet::new();
    for def in v2.rules {
        if !def.enabled {
            rules.entry(def.rule.clone()).or_insert_with(|| RuleConfig {
                enabled: false,
                options: def.options.clone(),
            });
            continue;
        }
        let valid_projects = def
            .projects
            .iter()
            .filter(|project| projects.contains_key(*project))
            .cloned()
            .collect::<Vec<_>>();
        let applies_to_repository = def.applies_to_repository();
        if !applies_to_repository && !def.projects.is_empty() && valid_projects.is_empty() {
            continue;
        }
        let entry = rules.entry(def.rule.clone()).or_insert_with(|| RuleConfig {
            enabled: true,
            options: def.options.clone(),
        });
        if !entry.enabled {
            entry.enabled = true;
            entry.options = def.options.clone();
        }
        if applies_to_repository {
            repository_rules.insert(def.rule.clone());
        }
        for project in valid_projects {
            if let Some(project) = projects.get_mut(&project) {
                project.rules.push(def.rule.clone());
            }
        }
    }
    Config {
        filesystem: FilesystemConfig {
            skip_directories: v2.filesystem.skip_directories,
        },
        projects,
        repository_rules,
        rules,
    }
}
