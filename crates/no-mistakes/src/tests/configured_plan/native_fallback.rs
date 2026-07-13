use super::relative_path;
use crate::tests::configured_plan_candidates::{
    group_candidates, merge_selected, selected_from_paths, stable_take, CoverageHints,
};
use crate::tests::{SelectedTest, TestFramework, Warning};
use no_mistakes::codebase::dependencies::graph::DepGraph;
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use no_mistakes::config::v2::schema::TestPlanGroupType;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub(super) fn native_traceable_changed_files(
    framework: TestFramework,
    root: &Path,
    changed_files: &[PathBuf],
    graph: &DepGraph,
    all_tests: &[PathBuf],
    all_test_set: &HashSet<PathBuf>,
    test_filter: &TestFileFilter,
    coverage_hints: &CoverageHints,
) -> HashSet<String> {
    if !matches!(framework, TestFramework::Dotnet | TestFramework::Swift) {
        return HashSet::new();
    }
    let mut warnings: Vec<Warning> = Vec::new();
    let mut warnings_seen = HashSet::new();
    group_candidates(
        TestPlanGroupType::Dependencies,
        root,
        changed_files,
        graph,
        all_tests,
        all_test_set,
        test_filter,
        &HashSet::new(),
        coverage_hints,
        &mut warnings,
        &mut warnings_seen,
    )
    .into_iter()
    .flat_map(|test| test.reasons.into_iter().map(|reason| reason.changed_file))
    .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn native_fallback_selection(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
    selected_map: &BTreeMap<PathBuf, SelectedTest>,
    extra_traced_changed_files: &HashSet<String>,
    used: &HashSet<String>,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    visible_paths: &[PathBuf],
    allow_full_suite_fallback: bool,
    limit: usize,
) -> Option<(String, Vec<SelectedTest>)> {
    let triggers = untraced_native_changes(
        framework,
        root,
        config,
        changed_files,
        deleted_files,
        selected_map,
        extra_traced_changed_files,
    );
    if triggers.is_empty() {
        return None;
    }

    let mut native_candidates: BTreeMap<String, SelectedTest> = BTreeMap::new();
    for trigger_file in &triggers {
        let fallback_tests = native_fallback_tests(
            framework,
            root,
            config,
            trigger_file,
            all_tests,
            discovered,
            visible_paths,
            allow_full_suite_fallback,
        );
        for candidate in selected_from_paths(
            root,
            &fallback_tests,
            "native source fallback",
            Some(trigger_file),
        ) {
            if used.contains(&candidate.test_file) {
                continue;
            }
            native_candidates
                .entry(candidate.test_file.clone())
                .and_modify(|entry| merge_selected(entry, &candidate))
                .or_insert(candidate);
        }
    }
    if native_candidates.is_empty() {
        return None;
    }

    let trigger_list = triggers
        .iter()
        .map(|trigger| format!("`{}`", relative_path(root, trigger)))
        .collect::<Vec<_>>()
        .join(", ");
    let reason = format!(
        "{} source impact for {} could not be determined; falling back to discovered {} tests",
        framework_name(framework),
        trigger_list,
        framework_name(framework)
    );
    let picked = stable_take(native_candidates.into_values().collect(), limit);
    Some((reason, picked))
}

pub(super) fn untraced_native_changes(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
    selected_map: &BTreeMap<PathBuf, SelectedTest>,
    extra_traced_changed_files: &HashSet<String>,
) -> Vec<PathBuf> {
    if !matches!(framework, TestFramework::Dotnet | TestFramework::Swift) {
        return Vec::new();
    }

    let mut traced_changed_files: HashSet<String> = selected_map
        .values()
        .flat_map(|test| {
            test.reasons
                .iter()
                .map(|reason| slash_path(&reason.changed_file))
        })
        .collect();
    traced_changed_files.extend(extra_traced_changed_files.iter().cloned());

    let mut triggers = BTreeSet::new();
    for changed in changed_files.iter().chain(deleted_files.iter()) {
        let rel = slash_path(&relative_path(root, changed));
        if is_native_source_or_project_change(framework, root, config, &rel)
            && !traced_changed_files.contains(&rel)
        {
            triggers.insert(changed.clone());
        }
    }
    triggers.into_iter().collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn native_fallback_tests(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    visible_paths: &[PathBuf],
    allow_full_suite_fallback: bool,
) -> Vec<PathBuf> {
    let scoped = match framework {
        TestFramework::Swift => {
            swift_manifest_fallback_tests(root, trigger_file, all_tests, discovered)
        }
        TestFramework::Dotnet => dotnet_fallback_tests(
            root,
            config,
            trigger_file,
            all_tests,
            discovered,
            visible_paths,
        ),
        TestFramework::Playwright | TestFramework::Vitest => Vec::new(),
    };
    if scoped.is_empty() && allow_full_suite_fallback {
        all_tests.to_vec()
    } else {
        scoped
    }
}

fn is_native_source_or_project_change(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    rel: &str,
) -> bool {
    let rel = slash_path(rel);
    match framework {
        TestFramework::Dotnet => {
            rel.ends_with(".csproj")
                || is_configured_dotnet_solution(root, config, &rel)
                || (rel.ends_with(".cs") && !is_dotnet_test_path(&rel))
        }
        TestFramework::Swift => {
            rel.ends_with("Package.swift")
                || (rel.ends_with(".swift")
                    && !rel
                        .split('/')
                        .any(|part| part.eq_ignore_ascii_case("tests")))
        }
        TestFramework::Playwright | TestFramework::Vitest => false,
    }
}

fn is_configured_dotnet_solution(root: &Path, config: &NoMistakesConfig, rel: &str) -> bool {
    rel.ends_with(".sln")
        && config.tests.dotnet.solutions.iter().any(|solution| {
            slash_path(&relative_path(root, &root.join(slash_path(solution)))) == rel
        })
}

fn is_dotnet_test_path(rel: &str) -> bool {
    slash_path(rel).split('/').any(|part| {
        part == "Tests" || part == "tests" || part.ends_with(".Tests") || part.ends_with(".Test")
    })
}

fn swift_manifest_fallback_tests(
    root: &Path,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
) -> Vec<PathBuf> {
    let rel = slash_path(&relative_path(root, trigger_file));
    if !rel.ends_with("Package.swift") {
        return Vec::new();
    }
    let package = if rel == "Package.swift" {
        ""
    } else {
        let Some(package) = rel.strip_suffix("/Package.swift") else {
            return Vec::new();
        };
        package
    };
    tests_with_target_configs(all_tests, discovered, [package.to_string()])
}

fn explicit_dotnet_test_projects(root: &Path, config: &NoMistakesConfig) -> BTreeSet<PathBuf> {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    config
        .tests
        .dotnet
        .projects
        .values()
        .filter(|project| project.test)
        .map(|project| {
            no_mistakes::codebase::ts_resolver::normalize_path(&root.join(&project.project))
        })
        .collect()
}

fn dotnet_project_is_test(
    facts: &no_mistakes::codebase::dotnet::DotnetProjectFacts,
    explicit_test_projects: &BTreeSet<PathBuf>,
) -> bool {
    facts.is_test || explicit_test_projects.contains(&facts.project_path)
}

fn dotnet_project_config(root: &Path, project_path: &Path) -> String {
    slash_path(&relative_path(root, project_path))
}

fn dotnet_project_fallback_tests(
    root: &Path,
    config: &NoMistakesConfig,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    visible_paths: &[PathBuf],
) -> Vec<PathBuf> {
    let rel = relative_path(root, trigger_file);
    let trigger = no_mistakes::codebase::ts_resolver::normalize_path(&root.join(&rel));
    let configured = no_mistakes::codebase::dotnet::configured_projects(root, &config.tests.dotnet);
    let facts =
        no_mistakes::codebase::dotnet::collect_dotnet_facts(root, visible_paths, &configured);
    if facts.projects.is_empty() {
        return Vec::new();
    }
    let explicit_test_projects = explicit_dotnet_test_projects(root, config);

    let mut reverse_refs: BTreeMap<PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
    for project in facts.projects.values() {
        for reference in &project.project_references {
            reverse_refs
                .entry(reference.clone())
                .or_default()
                .insert(project.project_path.clone());
        }
    }

    let mut queue = VecDeque::from([trigger.clone()]);
    let mut visited = BTreeSet::new();
    let mut test_project_configs = BTreeSet::new();
    while let Some(project_path) = queue.pop_front() {
        if !visited.insert(project_path.clone()) {
            continue;
        }
        if facts
            .projects
            .get(&project_path)
            .is_some_and(|project| dotnet_project_is_test(project, &explicit_test_projects))
        {
            test_project_configs.insert(dotnet_project_config(root, &project_path));
        }
        if let Some(referencing_projects) = reverse_refs.get(&project_path) {
            queue.extend(referencing_projects.iter().cloned());
        }
    }

    if test_project_configs.is_empty() {
        return Vec::new();
    }

    tests_with_nonempty_target_configs(all_tests, discovered, test_project_configs)
}

fn dotnet_fallback_tests(
    root: &Path,
    config: &NoMistakesConfig,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    visible_paths: &[PathBuf],
) -> Vec<PathBuf> {
    let rel = relative_path(root, trigger_file);
    if rel.ends_with(".csproj") {
        return dotnet_project_fallback_tests(
            root,
            config,
            trigger_file,
            all_tests,
            discovered,
            visible_paths,
        );
    }
    if rel.ends_with(".sln") {
        return dotnet_solution_fallback_tests(root, trigger_file, all_tests, discovered);
    }
    Vec::new()
}

fn dotnet_solution_fallback_tests(
    root: &Path,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
) -> Vec<PathBuf> {
    let rel = relative_path(root, trigger_file);
    if !rel.ends_with(".sln") {
        return Vec::new();
    }

    let solution_path = if trigger_file.is_absolute() {
        trigger_file.to_path_buf()
    } else {
        root.join(trigger_file)
    };
    let Ok(source) = std::fs::read_to_string(&solution_path) else {
        return Vec::new();
    };
    let solution_dir = solution_path.parent().unwrap_or(root);
    let project_configs = parse_solution_projects(root, solution_dir, &source);
    if project_configs.is_empty() {
        return Vec::new();
    }

    tests_with_target_configs(all_tests, discovered, project_configs)
}

fn parse_solution_projects(root: &Path, solution_dir: &Path, source: &str) -> Vec<String> {
    let re =
        regex::Regex::new(r#"(?m)^Project\("\{[^"]+\}"\)\s*=\s*"([^"]+)",\s*"([^"]+\.csproj)""#)
            .expect("valid regex");
    re.captures_iter(source)
        .filter_map(|cap| {
            let project_path = no_mistakes::codebase::ts_resolver::normalize_path(
                &solution_dir.join(PathBuf::from(cap.get(2)?.as_str().replace('\\', "/"))),
            );
            Some(relative_path(root, &project_path))
        })
        .collect()
}

fn tests_with_target_configs<I>(
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    configs: I,
) -> Vec<PathBuf>
where
    I: IntoIterator<Item = String>,
{
    let configs: BTreeSet<String> = configs
        .into_iter()
        .map(|config| normalize_config_path(&config))
        .collect();
    if configs.is_empty() {
        return Vec::new();
    }
    tests_with_target_config_set(all_tests, discovered, &configs)
}

fn tests_with_nonempty_target_configs<I>(
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    configs: I,
) -> Vec<PathBuf>
where
    I: IntoIterator<Item = String>,
{
    let configs: BTreeSet<String> = configs
        .into_iter()
        .map(|config| normalize_config_path(&config))
        .collect();
    if configs.is_empty() {
        return Vec::new();
    }
    tests_with_target_config_set(all_tests, discovered, &configs)
}

fn tests_with_target_config_set(
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    configs: &BTreeSet<String>,
) -> Vec<PathBuf> {
    all_tests
        .iter()
        .filter(|test| {
            discovered
                .targets_by_path
                .get(*test)
                .is_some_and(|targets| {
                    targets.iter().any(|target| {
                        target
                            .config
                            .as_ref()
                            .is_none_or(|config| configs.contains(&normalize_config_path(config)))
                    })
                })
        })
        .cloned()
        .collect()
}

fn slash_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn normalize_config_path(path: &str) -> String {
    let mut path = slash_path(path);
    while let Some(rest) = path.strip_prefix("./") {
        path = rest.to_string();
    }
    if path == "." {
        String::new()
    } else {
        path
    }
}

fn framework_name(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Playwright => "playwright",
        TestFramework::Vitest => "vitest",
        TestFramework::Dotnet => "dotnet",
        TestFramework::Swift => "swift",
    }
}

#[cfg(test)]
mod tests;
