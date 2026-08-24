use super::{
    relative_path, slash_path, tests_with_nonempty_target_configs, tests_with_target_configs,
};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

pub(super) fn is_configured_solution(root: &Path, config: &NoMistakesConfig, rel: &str) -> bool {
    rel.ends_with(".sln")
        && config.tests.dotnet.solutions.iter().any(|solution| {
            slash_path(&relative_path(root, &root.join(slash_path(solution)))) == rel
        })
}

pub(super) fn is_test_path(rel: &str) -> bool {
    slash_path(rel).split('/').any(|part| {
        part == "Tests" || part == "tests" || part.ends_with(".Tests") || part.ends_with(".Test")
    })
}

pub(super) fn explicit_test_projects(root: &Path, config: &NoMistakesConfig) -> BTreeSet<PathBuf> {
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

pub(super) fn project_is_test(
    facts: &no_mistakes::codebase::dotnet::DotnetProjectFacts,
    explicit_test_projects: &BTreeSet<PathBuf>,
) -> bool {
    facts.is_test || explicit_test_projects.contains(&facts.project_path)
}

fn project_config(root: &Path, project_path: &Path) -> String {
    slash_path(&relative_path(root, project_path))
}

pub(super) fn project_fallback_tests(
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
    let explicit_test_projects = explicit_test_projects(root, config);

    let mut reverse_refs: BTreeMap<PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
    for project in facts.projects.values() {
        for reference in &project.project_references {
            reverse_refs
                .entry(reference.clone())
                .or_default()
                .insert(project.project_path.clone());
        }
    }

    let mut queue = VecDeque::from([trigger]);
    let mut visited = BTreeSet::new();
    let mut test_project_configs = BTreeSet::new();
    while let Some(project_path) = queue.pop_front() {
        if !visited.insert(project_path.clone()) {
            continue;
        }
        if facts
            .projects
            .get(&project_path)
            .is_some_and(|project| project_is_test(project, &explicit_test_projects))
        {
            test_project_configs.insert(project_config(root, &project_path));
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

pub(super) fn solution_fallback_tests(
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
