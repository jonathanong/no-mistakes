use super::super::test_flake_helpers::{
    compile_optional_globs, configured_paths, is_playwright_config_name, is_vitest_config_name,
    AllowEntry, CompiledAllowList,
};
use super::RULE_ID;
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeSet;

const DEFAULT_RETRY_ACTIONS: &[&str] = &["nick-fields/retry", "wandalen/wretry.action"];

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    pub(super) max_retries: u64,
    pub(super) allow: Vec<AllowEntry>,
    pub(super) files: Vec<String>,
    pub(super) configs: Vec<String>,
    pub(super) retry_actions: Vec<String>,
    pub(super) ci_test_commands: Vec<String>,
}

pub(super) struct CompiledOptions {
    pub(super) max_retries: u64,
    pub(super) allow: CompiledAllowList,
    pub(super) files: Option<GlobMatcher>,
    configs: Option<GlobMatcher>,
    vitest_configs: BTreeSet<String>,
    playwright_configs: BTreeSet<String>,
    retry_actions: Vec<String>,
    pub(super) ci_test_commands: Vec<Vec<String>>,
}

impl CompiledOptions {
    pub(super) fn is_playwright_config(&self, rel: &str) -> bool {
        self.playwright_configs.contains(rel)
            || self.matches_explicit_config(rel)
            || is_playwright_config_name(rel)
    }

    pub(super) fn is_vitest_config(&self, rel: &str) -> bool {
        self.vitest_configs.contains(rel)
            || self.matches_explicit_config(rel)
            || is_vitest_config_name(rel)
    }

    fn matches_explicit_config(&self, rel: &str) -> bool {
        self.configs
            .as_ref()
            .is_some_and(|matcher| matcher.is_match(rel))
    }

    pub(super) fn matches_retry_action(&self, uses: &str) -> bool {
        let base = uses
            .trim()
            .split('@')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        self.retry_actions.iter().any(|action| action == &base)
    }
}

pub(super) fn compile(opts: Options, config: &NoMistakesConfig) -> Result<CompiledOptions> {
    let mut retry_actions: Vec<String> = DEFAULT_RETRY_ACTIONS
        .iter()
        .map(|item| (*item).to_string())
        .collect();
    for action in opts.retry_actions {
        let trimmed = action.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            bail!("{RULE_ID} retryActions entries must be non-empty");
        }
        if !retry_actions.contains(&trimmed) {
            retry_actions.push(trimmed);
        }
    }
    let ci_test_commands = collect_ci_test_commands(config, &opts.ci_test_commands)?;
    Ok(CompiledOptions {
        max_retries: opts.max_retries,
        allow: CompiledAllowList::compile(RULE_ID, &opts.allow)?,
        files: compile_optional_globs(&opts.files, &format!("{RULE_ID} files"))?,
        configs: compile_optional_globs(&opts.configs, &format!("{RULE_ID} configs"))?,
        vitest_configs: configured_paths(&config.tests.vitest.configs),
        playwright_configs: configured_paths(&config.tests.playwright.configs),
        retry_actions,
        ci_test_commands,
    })
}

fn collect_ci_test_commands(
    config: &NoMistakesConfig,
    explicit: &[String],
) -> Result<Vec<Vec<String>>> {
    let mut commands = Vec::new();
    let tests = &config.tests;
    if tests.vitest.configs.is_some() || !tests.vitest.projects.is_empty() {
        push_cmd(&mut commands, &["vitest"]);
    }
    if tests.playwright.configs.is_some()
        || !tests.playwright.projects.is_empty()
        || !tests.playwright.test_include.is_empty()
    {
        push_cmd(&mut commands, &["playwright"]);
    }
    if tests.jest.configs.is_some() || !tests.jest.projects.is_empty() {
        push_cmd(&mut commands, &["jest"]);
    }
    if !tests.rust.packages.is_empty() {
        push_cmd(&mut commands, &["cargo", "test"]);
        push_cmd(&mut commands, &["cargo", "nextest"]);
    }
    if !tests.go.modules.is_empty() {
        push_cmd(&mut commands, &["go", "test"]);
    }
    if !tests.python.packages.is_empty() {
        push_cmd(&mut commands, &["pytest"]);
    }
    if !tests.swift.packages.is_empty()
        || !tests.swift.projects.is_empty()
        || !tests.swift.base_commands.is_empty()
    {
        push_cmd(&mut commands, &["swift", "test"]);
        for argv in tests.swift.base_commands.values() {
            let tokens: Vec<String> = argv
                .iter()
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
            if !tokens.is_empty() && !commands.contains(&tokens) {
                commands.push(tokens);
            }
        }
    }
    if !tests.dotnet.solutions.is_empty() || !tests.dotnet.projects.is_empty() {
        push_cmd(&mut commands, &["dotnet", "test"]);
    }
    if !tests.rails.apps.is_empty() {
        push_cmd(&mut commands, &["rails", "test"]);
        push_cmd(&mut commands, &["rspec"]);
    }
    if !tests.php.apps.is_empty() || tests.php.framework.is_some() {
        push_cmd(&mut commands, &["phpunit"]);
        push_cmd(&mut commands, &["pest"]);
    }
    for raw in explicit {
        let tokens: Vec<String> = raw.split_whitespace().map(str::to_string).collect();
        if tokens.is_empty() {
            bail!("{RULE_ID} ciTestCommands entries must be non-empty");
        }
        if !commands.contains(&tokens) {
            commands.push(tokens);
        }
    }
    Ok(commands)
}

fn push_cmd(out: &mut Vec<Vec<String>>, parts: &[&str]) {
    let tokens: Vec<String> = parts.iter().map(|part| (*part).to_string()).collect();
    if !out.contains(&tokens) {
        out.push(tokens);
    }
}
