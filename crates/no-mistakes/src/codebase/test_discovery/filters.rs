use super::types::TestRunner;
use crate::integration_tests::types::ConfigProject;

pub use crate::codebase::testing_analysis::project_filter::{
    fallback_test_path, ProjectTestFilter,
};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

pub(crate) fn compile_project_filter(project: &ConfigProject) -> anyhow::Result<ProjectTestFilter> {
    ProjectTestFilter::from_patterns(&project.include, &project.exclude)
}

pub(crate) fn fallback_runner_match(runner: TestRunner, rel: &str) -> bool {
    match runner {
        TestRunner::Dotnet => {
            rel.ends_with(".cs")
                && (rel.contains("/Tests/")
                    || rel.contains(".Tests/")
                    || rel.contains("/tests/")
                    || rel.contains(".Test/"))
        }
        TestRunner::Vitest => {
            fallback_test_path(rel)
                && !rel.split('/').any(|component| component == "playwright")
                && !has_path_segment_pair(rel, "tests", "e2e")
                && !rel.starts_with("specs/")
        }
        TestRunner::Playwright => {
            fallback_test_path(rel)
                && (rel.contains("/tests/e2e/")
                    || rel.starts_with("tests/e2e/")
                    || rel.contains("/playwright/")
                    || rel.starts_with("playwright/")
                    || rel.starts_with("specs/"))
        }
        TestRunner::Swift => rel.contains("/Tests/") && rel.ends_with(".swift"),
        TestRunner::Python => is_python_test_path(rel),
        TestRunner::Go => rel.ends_with("_test.go"),
        TestRunner::Cargo => is_cargo_test_path(rel),
        TestRunner::Rails => rel.ends_with("_spec.rb") || rel.ends_with("_test.rb"),
        TestRunner::Php => rel.ends_with("Test.php") || rel.contains("/tests/"),
        TestRunner::Java => is_java_test_path(rel),
        TestRunner::Kotlin => is_kotlin_test_path(rel),
        TestRunner::Elixir => is_elixir_test_path(rel),
        TestRunner::Dart => is_dart_test_path(rel),
        TestRunner::Jest => fallback_test_path(rel),
    }
}

fn is_python_test_path(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.starts_with("test_") && name.ends_with(".py")
        || name.ends_with("_test.py")
        || name == "tests.py"
        || rel.contains("/tests/") && name.ends_with(".py")
}

fn is_cargo_test_path(rel: &str) -> bool {
    let slash = rel.replace('\\', "/");
    slash.contains("/tests/") && slash.ends_with(".rs")
        || slash.ends_with("/tests.rs")
        || slash.ends_with("_test.rs")
}

fn is_java_test_path(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.ends_with("Test.java") || name.ends_with("Tests.java") || name.ends_with("IT.java")
}

fn is_kotlin_test_path(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.ends_with("Test.kt") || name.ends_with("Tests.kt") || name.ends_with("IT.kt")
}

fn is_elixir_test_path(rel: &str) -> bool {
    rel.rsplit('/').next().unwrap_or(rel).ends_with("_test.exs")
}

fn is_dart_test_path(rel: &str) -> bool {
    rel.rsplit('/')
        .next()
        .unwrap_or(rel)
        .ends_with("_test.dart")
}

fn has_path_segment_pair(path: &str, first: &str, second: &str) -> bool {
    let mut segments = path.split('/');
    let mut previous = segments.next().unwrap_or("");
    for current in segments {
        if previous == first && current == second {
            return true;
        }
        previous = current;
    }
    false
}
