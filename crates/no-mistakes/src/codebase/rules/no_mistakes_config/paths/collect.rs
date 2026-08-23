use super::{Kind, Ref};
use crate::codebase::workflow_topology::posix_path::normalize;
use crate::config::v2::schema::{StringOrList, TestPlanFrameworkConfig, TestPlanProjectDependency};
use crate::config::v2::NoMistakesConfig;
use serde_yaml::Value;

pub(super) fn collect(config: &NoMistakesConfig, refs: &mut Vec<Ref>) {
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
    collect_rule_options(config, refs);
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
        for (project_name, dependency) in &plan.full_suite_triggers.projects {
            let root = config
                .projects
                .get(project_name)
                .and_then(|project| project.root.as_deref())
                .unwrap_or("");
            let paths = match dependency {
                TestPlanProjectDependency::Patterns(paths) => paths,
                TestPlanProjectDependency::Targeted(targeted) => &targeted.paths,
                TestPlanProjectDependency::All(_) => continue,
            };
            for (path_index, path) in paths.iter().enumerate() {
                let Some(path) = project_relative(root, path) else {
                    continue;
                };
                push(
                    refs,
                    format!(
                        "testPlan.{framework}.fullSuiteTriggers.projects.{project_name}[{path_index}]"
                    ),
                    path_kind(&path),
                    &path,
                );
            }
        }
        for (index, trigger) in plan.full_suite_triggers.triggers.iter().enumerate() {
            for (path_index, path) in trigger.paths.iter().enumerate() {
                if path.starts_with('!') {
                    continue;
                }
                push(
                    refs,
                    format!(
                        "testPlan.{framework}.fullSuiteTriggers.triggers[{index}].paths[{path_index}]"
                    ),
                    path_kind(path),
                    path,
                );
            }
        }
    }
}

fn collect_rule_options(config: &NoMistakesConfig, refs: &mut Vec<Ref>) {
    for (index, rule) in config.rules.iter().enumerate() {
        collect_option_paths(&rule.options, &format!("rules[{index}].options"), refs);
    }
}

fn collect_option_paths(value: &Value, field: &str, refs: &mut Vec<Ref>) {
    let Some(map) = value.as_mapping() else {
        return;
    };
    for (key, value) in map {
        let Some(key) = key.as_str() else {
            continue;
        };
        let child = format!("{field}.{key}");
        match key {
            "tsconfig" | "lockfile" | "shellFiles" | "allowlist" => {
                for (index, path) in string_values(value).into_iter().enumerate() {
                    if let Some(path) = required_path(&path) {
                        push(refs, format!("{child}[{index}]"), Kind::File, &path);
                    }
                }
            }
            "roots" | "selectorRoots" | "shebangDirs" => {
                for (index, path) in string_values(value).into_iter().enumerate() {
                    if let Some(path) = required_path(&path) {
                        push(
                            refs,
                            format!("{child}[{index}]"),
                            if source_file_path(&path) {
                                Kind::File
                            } else {
                                Kind::Directory
                            },
                            &path,
                        );
                    }
                }
            }
            "packages" => {
                if let Some(packages) = value.as_sequence() {
                    for (index, package) in packages.iter().enumerate() {
                        let Some(root) = package.get("root").and_then(Value::as_str) else {
                            continue;
                        };
                        if let Some(root) = required_path(root) {
                            push(
                                refs,
                                format!("{child}[{index}].root"),
                                Kind::Directory,
                                &root,
                            );
                        }
                    }
                }
            }
            // Exclusions are deliberately not validated: a defensive exclude may
            // refer to a path that is absent until a later feature is introduced.
            "excludePaths" | "conditionallyAllowedWorkflows" => {}
            _ => collect_option_paths(value, &child, refs),
        }
    }
}

fn string_values(value: &Value) -> Vec<String> {
    match value {
        Value::String(value) => vec![value.clone()],
        Value::Sequence(values) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn required_path(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && !value.starts_with('!') && !path_kind(value).eq(&Kind::Glob))
        .then(|| value.to_string())
}

fn source_file_path(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        ".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mts", ".cts", ".json",
    ]
    .iter()
    .any(|suffix| value.ends_with(suffix))
}

fn path_kind(value: &str) -> Kind {
    if value.contains('*') || value.contains('?') || value.contains('{') {
        Kind::Glob
    } else {
        Kind::File
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

pub(crate) fn frameworks(
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
    if !value.is_empty() && !value.starts_with('!') {
        refs.push(Ref {
            field,
            kind,
            value: value.to_string(),
        });
    }
}
