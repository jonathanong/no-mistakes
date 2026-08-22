use super::native_fallback::slash_path;
use super::relative_path;
use crate::tests::TestFramework;
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn is_language_native_change(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    rel: &str,
) -> bool {
    let rel = slash_path(rel);
    match framework {
        TestFramework::Python => is_python_native(&rel, root, config),
        TestFramework::Go => is_go_native(&rel, root, config),
        TestFramework::Cargo => is_cargo_native(&rel, root, config),
        TestFramework::Rails => is_rails_native(&rel, root, config),
        TestFramework::Php => is_php_native(&rel, root, config),
        TestFramework::Java => is_java_native(&rel, root, config),
        TestFramework::Kotlin => is_kotlin_native(&rel, root, config),
        _ => false,
    }
}

pub(super) fn language_fallback_tests(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    trigger_file: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
) -> Vec<PathBuf> {
    let owner = owning_root(framework, root, config, trigger_file);
    let Some(owner) = owner else {
        return Vec::new();
    };
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
                            .as_deref()
                            .is_some_and(|config| normalize_root(config) == owner)
                    })
                })
        })
        .cloned()
        .collect()
}

fn is_python_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.python.packages)
        && (is_python_manifest(rel) || (rel.ends_with(".py") && !is_python_test(rel)))
        || is_named_manifest(root, &config.tests.python.packages, rel, "pyproject.toml")
}

fn is_go_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.go.modules)
        && (rel.ends_with("go.mod") || (rel.ends_with(".go") && !rel.ends_with("_test.go")))
        || is_named_manifest(root, &config.tests.go.modules, rel, "go.mod")
}

fn is_cargo_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.rust.packages)
        && (rel.ends_with("Cargo.toml") || (rel.ends_with(".rs") && !is_cargo_test(rel)))
        || is_named_manifest(root, &config.tests.rust.packages, rel, "Cargo.toml")
}

fn is_rails_native(rel: &str, _root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.rails.apps)
        && (rel.ends_with("Gemfile")
            || (rel.ends_with(".rb") && !rel.ends_with("_spec.rb") && !rel.ends_with("_test.rb")))
}

fn is_php_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.php.apps)
        && (rel.ends_with("composer.json")
            || (rel.ends_with(".php") && !rel.ends_with("Test.php") && !rel.contains("/tests/")))
        || is_named_manifest(root, &config.tests.php.apps, rel, "composer.json")
}

fn is_java_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.java.packages)
        && (rel.ends_with("pom.xml")
            || rel.ends_with("build.gradle")
            || rel.ends_with("build.gradle.kts")
            || (rel.ends_with(".java") && !is_java_test(rel)))
        || is_named_manifest(root, &config.tests.java.packages, rel, "pom.xml")
}

include!("native_fallback_lang_kotlin.rs");

fn owning_root(
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    trigger_file: &Path,
) -> Option<String> {
    let rel = slash_path(&relative_path(root, trigger_file));
    let roots = configured_roots(framework, config);
    roots
        .iter()
        .map(|entry| normalize_root(entry))
        .filter(|entry| {
            entry.is_empty()
                || entry == "."
                || rel == *entry
                || rel.starts_with(&format!("{entry}/"))
        })
        .max_by_key(|entry| entry.len())
}

fn configured_roots(framework: TestFramework, config: &NoMistakesConfig) -> &[String] {
    match framework {
        TestFramework::Python => &config.tests.python.packages,
        TestFramework::Go => &config.tests.go.modules,
        TestFramework::Cargo => &config.tests.rust.packages,
        TestFramework::Rails => &config.tests.rails.apps,
        TestFramework::Php => &config.tests.php.apps,
        TestFramework::Java => &config.tests.java.packages,
        TestFramework::Kotlin => &config.tests.kotlin.packages,
        _ => &[],
    }
}

fn under_roots(rel: &str, roots: &[String]) -> bool {
    roots.iter().any(|root| {
        let root = normalize_root(root);
        root.is_empty() || root == "." || rel == root || rel.starts_with(&format!("{root}/"))
    })
}

fn is_named_manifest(root: &Path, roots: &[String], rel: &str, name: &str) -> bool {
    roots.iter().any(|entry| {
        let entry = normalize_root(entry);
        let expected = if entry.is_empty() || entry == "." {
            name.to_string()
        } else {
            format!("{entry}/{name}")
        };
        slash_path(&relative_path(root, &root.join(&expected))) == rel || rel == expected
    })
}

fn is_python_test(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.starts_with("test_") && name.ends_with(".py")
        || name.ends_with("_test.py")
        || name == "tests.py"
        || rel.contains("/tests/") && name.ends_with(".py")
}

fn is_java_test(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.ends_with("Test.java")
        || name.ends_with("Tests.java")
        || name.ends_with("IT.java")
        || (rel.contains("/src/test/") || rel.starts_with("src/test/")) && name.ends_with(".java")
}

fn is_cargo_test(rel: &str) -> bool {
    rel.contains("/tests/") && rel.ends_with(".rs")
        || rel.ends_with("/tests.rs")
        || rel.ends_with("_test.rs")
}

fn is_python_manifest(rel: &str) -> bool {
    rel.ends_with("pyproject.toml") || rel.ends_with("setup.cfg") || rel.ends_with("setup.py")
}

fn normalize_root(value: &str) -> String {
    let mut path = slash_path(value);
    while let Some(rest) = path.strip_prefix("./") {
        path = rest.to_string();
    }
    path.trim_end_matches('/').to_string()
}

#[cfg(test)]
#[path = "native_fallback_lang/tests.rs"]
mod tests;
