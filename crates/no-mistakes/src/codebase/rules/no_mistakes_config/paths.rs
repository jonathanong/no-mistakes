use super::finding;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::{StringOrList, TestPlanFrameworkConfig};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
enum Kind {
    File,
    Directory,
    Glob,
}

struct Ref {
    field: String,
    kind: Kind,
    value: String,
}

pub(super) fn lint(
    config: &NoMistakesConfig,
    tracked: &BTreeSet<String>,
    config_file: &str,
) -> Result<Vec<RuleFinding>> {
    let mut refs = Vec::new();
    collect(config, &mut refs);
    Ok(refs
        .into_iter()
        .filter_map(|item| missing(item, tracked, config_file))
        .collect())
}

fn missing(item: Ref, tracked: &BTreeSet<String>, config_file: &str) -> Option<RuleFinding> {
    let present = match item.kind {
        Kind::Glob => glob_matches(&item.value, tracked),
        Kind::File => tracked.contains(item.value.trim_start_matches("./")),
        Kind::Directory => directory_present(&item.value, tracked),
    };
    (!present).then(|| {
        finding(
            config_file,
            format!(
                "{}: missing {} `{}`",
                item.field,
                kind_label(item.kind),
                item.value
            ),
        )
    })
}

fn kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::File => "file",
        Kind::Directory => "directory",
        Kind::Glob => "path",
    }
}

fn directory_present(value: &str, tracked: &BTreeSet<String>) -> bool {
    let prefix = value.trim_start_matches("./").trim_end_matches('/');
    tracked.contains(prefix)
        || tracked
            .iter()
            .any(|rel| rel == prefix || rel.starts_with(&format!("{prefix}/")))
}

fn glob_matches(pattern: &str, tracked: &BTreeSet<String>) -> bool {
    crate::codebase::rules::path_filter::GlobMatcher::new(
        &[pattern.to_string()],
        "no-mistakes-config path",
    )
    .map(|matcher| tracked.iter().any(|rel| matcher.is_match(rel)))
    .unwrap_or(false)
}

fn collect(config: &NoMistakesConfig, refs: &mut Vec<Ref>) {
    push_opt(
        refs,
        "frontendRoot",
        Kind::Directory,
        config.frontend_root.as_deref(),
    );
    for (name, project) in &config.projects {
        push_opt(
            refs,
            &format!("projects.{name}.root"),
            Kind::Directory,
            project.root.as_deref(),
        );
    }
    collect_tests(config, refs);
    collect_test_plan(config, refs);
}

fn collect_tests(config: &NoMistakesConfig, refs: &mut Vec<Ref>) {
    let playwright = &config.tests.playwright;
    push_list(
        refs,
        "tests.playwright.configs",
        Kind::File,
        &playwright.configs,
    );
    for (index, root) in playwright.selector_roots.iter().enumerate() {
        push(
            refs,
            format!("tests.playwright.selectorRoots[{index}]"),
            Kind::Directory,
            root,
        );
    }
    push_opt(
        refs,
        "tests.playwright.frontendRoot",
        Kind::Directory,
        playwright.frontend_root.as_deref(),
    );
    for (index, helper) in playwright.navigation_helpers.iter().enumerate() {
        push(
            refs,
            format!("tests.playwright.navigationHelpers[{index}]"),
            Kind::File,
            helper,
        );
    }
    push_list(
        refs,
        "tests.vitest.configs",
        Kind::File,
        &config.tests.vitest.configs,
    );
    push_list(
        refs,
        "tests.jest.configs",
        Kind::File,
        &config.tests.jest.configs,
    );
    push_list(
        refs,
        "tests.storybook.configs",
        Kind::File,
        &config.tests.storybook.configs,
    );
    for (index, package) in config.tests.swift.packages.iter().enumerate() {
        push(
            refs,
            format!("tests.swift.packages[{index}]"),
            Kind::Directory,
            package,
        );
    }
}

fn collect_test_plan(config: &NoMistakesConfig, refs: &mut Vec<Ref>) {
    for (framework, plan) in frameworks(config) {
        for (index, trigger) in plan.full_suite_triggers.triggers.iter().enumerate() {
            for (path_index, path) in trigger.paths.iter().enumerate() {
                let kind = if path.contains('*') {
                    Kind::Glob
                } else {
                    Kind::File
                };
                push(
                    refs,
                    format!("testPlan.{framework}.fullSuiteTriggers.triggers[{index}].paths[{path_index}]"),
                    kind,
                    path,
                );
            }
        }
    }
}

pub(super) fn frameworks(
    config: &NoMistakesConfig,
) -> [(&'static str, &TestPlanFrameworkConfig); 14] {
    [
        ("dotnet", &config.test_plan.dotnet),
        ("playwright", &config.test_plan.playwright),
        ("vitest", &config.test_plan.vitest),
        ("swift", &config.test_plan.swift),
        ("python", &config.test_plan.python),
        ("go", &config.test_plan.go),
        ("cargo", &config.test_plan.cargo),
        ("rails", &config.test_plan.rails),
        ("php", &config.test_plan.php),
        ("java", &config.test_plan.java),
        ("kotlin", &config.test_plan.kotlin),
        ("elixir", &config.test_plan.elixir),
        ("dart", &config.test_plan.dart),
        ("jest", &config.test_plan.jest),
    ]
}

fn push_list(refs: &mut Vec<Ref>, field: &str, kind: Kind, values: &Option<StringOrList>) {
    let Some(values) = values else {
        return;
    };
    for (index, value) in values.values().iter().enumerate() {
        push(refs, format!("{field}[{index}]"), kind, value);
    }
}

fn push_opt(refs: &mut Vec<Ref>, field: &str, kind: Kind, value: Option<&str>) {
    if let Some(value) = value {
        push(refs, field.to_string(), kind, value);
    }
}

fn push(refs: &mut Vec<Ref>, field: String, kind: Kind, value: &str) {
    if !value.is_empty() {
        refs.push(Ref {
            field,
            kind,
            value: value.to_string(),
        });
    }
}
