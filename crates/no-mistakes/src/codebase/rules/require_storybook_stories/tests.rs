// no-mistakes-disable-file rust-max-lines-per-file: legacy fixture-heavy test module
use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_resolver::{normalize_path, ImportResolver, TsConfig};
use crate::codebase::ts_symbols::{Export, ExportKind, FileSymbols};
use crate::config::v2::schema::{Project, ProjectType, StringOrList};
use crate::react_traits::report::types::{ComponentFacts, ComponentRef, Environment};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/require-storybook-stories/fixture")
        .join(name)
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            root: Some(".".to_string()),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        projects: vec!["web".to_string()],
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn config_with_storybook(options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.tests.storybook.configs = Some(StringOrList::One(".storybook/main.ts".to_string()));
    config
}

fn config_with_project_root(root: &str, options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.projects.get_mut("web").unwrap().root = Some(root.to_string());
    config
}

fn empty_resolver(root: &std::path::Path) -> ImportResolver<'static> {
    let tsconfig = Box::leak(Box::new(TsConfig {
        dir: root.to_path_buf(),
        paths: vec![],
        paths_dir: root.to_path_buf(),
        base_url: None,
    }));
    ImportResolver::new(tsconfig)
}

fn react_component(name: &str, file: &str, children: Vec<ComponentRef>) -> ComponentFacts {
    ComponentFacts {
        name: name.to_string(),
        file: file.to_string(),
        environment: Environment::Client,
        has_state: false,
        has_props: false,
        passes_props: false,
        uses_memo: false,
        uses_context_provider: false,
        uses_suspense: false,
        fetches: Vec::new(),
        dependencies: Vec::new(),
        children,
        inherited_from_children: None,
    }
}

fn react_facts(
    components: Vec<ComponentFacts>,
) -> crate::react_traits::analyze::file::FileAnalysis {
    crate::react_traits::analyze::file::FileAnalysis { components }
}

mod config_helpers;
mod coverage_helpers;
mod coverage_rule_cases;
mod selection_rule_cases;
