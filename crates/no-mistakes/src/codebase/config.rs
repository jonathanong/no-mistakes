use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::resolve;
use crate::config::v2::{find_config_root, load_v2_config};

#[path = "config/conversion.rs"]
mod conversion;
#[path = "config/discovery.rs"]
mod discovery;
#[path = "config/project.rs"]
mod project;
#[path = "config/rule_application.rs"]
mod rule_application;

pub use project::{
    infer_nextjs_root, infer_remix_root, infer_vitejs_root, InferredRoots, ProjectConfig,
};
pub use rule_application::RuleApplicationConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FilesystemConfig {
    #[serde(default)]
    pub skip_directories: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct RouteOptions {
    pub backend_pattern: String,
    pub backend_register_object: String,
    pub frontend_root: String,
    pub scan_patterns: Vec<String>,
    pub backend_prefixes: Vec<String>,
    pub backend_exact_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueueOptions {
    pub queue_pattern: String,
    pub factory_specifier: String,
    pub factory_function: String,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct HttpRouteOptions {
    pub backend_pattern: String,
    pub register_object: String,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct HttpCallOptions {
    pub backend_prefixes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct RuleConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(flatten, default)]
    pub options: serde_yaml::Value,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub filesystem: FilesystemConfig,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
    #[serde(default)]
    pub repository_rules: HashSet<String>,
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
    #[serde(default)]
    pub rule_applications: Vec<RuleApplicationConfig>,
}

impl Config {
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("failed to parse .guardrailsrc.yml")
    }

    pub fn rule_options<T: for<'de> Deserialize<'de> + Default>(&self, rule_id: &str) -> T {
        self.rules
            .get(rule_id)
            .and_then(|rule| serde_yaml::from_value(rule.options.clone()).ok())
            .unwrap_or_default()
    }

    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.rules
            .get(rule_id)
            .map(|rule| rule.enabled)
            .unwrap_or(true)
    }

    pub fn project_roots_for_rule(&self, root: &Path, rule_id: &str) -> Vec<PathBuf> {
        project::roots_for_rule(
            &self.projects,
            &self.rules,
            &self.repository_rules,
            root,
            rule_id,
        )
    }

    pub fn rule_applications_for(&self, rule_id: &str) -> Vec<&RuleApplicationConfig> {
        self.rule_applications
            .iter()
            .filter(|application| application.rule == rule_id)
            .collect()
    }

    pub fn project_roots_for_rule_application(
        &self,
        root: &Path,
        application: &RuleApplicationConfig,
    ) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        for project in application
            .projects
            .iter()
            .filter_map(|project_name| self.projects.get(project_name))
        {
            roots.push(
                project
                    .effective_root(root)
                    .unwrap_or_else(|| root.to_path_buf()),
            );
        }
        if application.repository {
            roots.push(root.to_path_buf());
        }
        roots.sort();
        roots.dedup();
        roots
    }

    pub fn augment_from_gitignore(&mut self, root: &Path) {
        let Ok(content) = std::fs::read_to_string(root.join(".gitignore")) else {
            return;
        };

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with('!')
                || trimmed.contains('/')
                || trimmed.contains('*')
                || trimmed.contains('?')
                || trimmed.contains('[')
            {
                continue;
            }

            let directory = trimmed.to_string();
            if !self.filesystem.skip_directories.contains(&directory) {
                self.filesystem.skip_directories.push(directory);
            }
        }
    }
}

pub fn load_config(start: &Path) -> Result<Config> {
    load_config_with_path(start, None)
}

pub fn load_config_with_path(start: &Path, config_path: Option<&Path>) -> Result<Config> {
    let v2 = load_v2_config(start, config_path)?;
    let mut config = conversion::config_from_v2(v2);
    let gitignore_root = match config_path {
        Some(path) => {
            let resolved = resolve(start, path);
            resolved
                .parent()
                .map(Path::to_path_buf)
                .expect("resolved config path has a parent")
        }
        None => find_config_root(start),
    };
    config.augment_from_gitignore(&gitignore_root);
    Ok(config)
}

pub fn load_codebase_config_with_path(start: &Path, config_path: Option<&Path>) -> Result<Config> {
    if config_path.is_some() {
        return load_config_with_path(start, config_path);
    }

    let Some(path) = discovery::find_codebase_config_path(start)? else {
        let mut config = Config::default();
        config.augment_from_gitignore(start);
        return Ok(config);
    };

    let v2 = load_v2_config(start, Some(&path))?;
    let mut config = conversion::config_from_v2(v2);
    config.augment_from_gitignore(path.parent().unwrap_or(start));
    Ok(config)
}

#[cfg(test)]
mod tests;
