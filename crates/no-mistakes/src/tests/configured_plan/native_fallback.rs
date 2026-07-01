use super::relative_path;
use crate::tests::{SelectedTest, TestFramework};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};

pub(super) fn untraced_native_change(
    framework: TestFramework,
    root: &Path,
    changed_files: &[PathBuf],
    selected_map: &BTreeMap<PathBuf, SelectedTest>,
) -> Option<PathBuf> {
    if !matches!(framework, TestFramework::Dotnet | TestFramework::Swift) {
        return None;
    }

    let traced_changed_files: HashSet<String> = selected_map
        .values()
        .flat_map(|test| {
            test.reasons
                .iter()
                .map(|reason| slash_path(&reason.changed_file))
        })
        .collect();

    changed_files.iter().find_map(|changed| {
        let rel = slash_path(&relative_path(root, changed));
        (is_native_source_or_project_change(framework, &rel)
            && !traced_changed_files.contains(&rel))
        .then(|| changed.clone())
    })
}

pub(super) fn native_fallback_tests(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
) -> Vec<PathBuf> {
    let scoped = match framework {
        TestFramework::Swift => {
            swift_manifest_fallback_tests(root, trigger_file, all_tests, discovered)
        }
        TestFramework::Dotnet => {
            dotnet_project_fallback_tests(root, config, trigger_file, all_tests, discovered)
        }
        TestFramework::Playwright | TestFramework::Vitest => Vec::new(),
    };
    if scoped.is_empty() {
        all_tests.to_vec()
    } else {
        scoped
    }
}

fn is_native_source_or_project_change(framework: TestFramework, rel: &str) -> bool {
    let rel = slash_path(rel);
    match framework {
        TestFramework::Dotnet => {
            rel.ends_with(".csproj") || (rel.ends_with(".cs") && !is_dotnet_test_path(&rel))
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
    let Some(package) = rel.strip_suffix("/Package.swift") else {
        return Vec::new();
    };
    tests_with_target_configs(all_tests, discovered, [package.to_string()])
}

fn dotnet_project_fallback_tests(
    root: &Path,
    config: &NoMistakesConfig,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
) -> Vec<PathBuf> {
    let rel = relative_path(root, trigger_file);
    if !rel.ends_with(".csproj") {
        return Vec::new();
    }

    let trigger = no_mistakes::codebase::ts_resolver::normalize_path(&root.join(&rel));
    let all_files =
        no_mistakes::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    let configured = no_mistakes::codebase::dotnet::configured_projects(root, &config.tests.dotnet);
    let facts = no_mistakes::codebase::dotnet::collect_dotnet_facts(root, &all_files, &configured);
    if facts.projects.is_empty() {
        return Vec::new();
    }

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
            .is_some_and(|project| project.is_test)
        {
            test_project_configs.insert(slash_path(&relative_path(root, &project_path)));
        }
        if let Some(referencing_projects) = reverse_refs.get(&project_path) {
            queue.extend(referencing_projects.iter().cloned());
        }
    }

    tests_with_target_configs(all_tests, discovered, test_project_configs)
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
        .map(|config| slash_path(&config))
        .collect();
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
                            .is_some_and(|config| configs.contains(&slash_path(config)))
                    })
                })
        })
        .cloned()
        .collect()
}

fn slash_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_source_detection_handles_backslash_paths() {
        assert!(is_native_source_or_project_change(
            TestFramework::Swift,
            r"swift-clients\core\Sources\App\Config.swift"
        ));
        assert!(!is_native_source_or_project_change(
            TestFramework::Swift,
            r"swift-clients\core\Tests\AppTests\ConfigTests.swift"
        ));
        assert!(!is_native_source_or_project_change(
            TestFramework::Swift,
            r"swift-clients\core\tests\AppTests\ConfigTests.swift"
        ));
        assert!(is_native_source_or_project_change(
            TestFramework::Dotnet,
            r"dotnet-clients\src\App\AppConfig.cs"
        ));
        assert!(!is_native_source_or_project_change(
            TestFramework::Dotnet,
            r"dotnet-clients\tests\App.Tests\AppConfigTests.cs"
        ));
    }
}
