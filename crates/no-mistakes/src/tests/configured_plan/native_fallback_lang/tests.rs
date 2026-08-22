use super::*;
use crate::config::v2::schema::NoMistakesConfig;
use crate::tests::TestFramework;
use std::path::PathBuf;

#[test]
fn python_source_under_configured_package_is_native() {
    let mut config = NoMistakesConfig::default();
    config.tests.python.packages = vec!["app".to_string()];
    assert!(is_language_native_change(
        TestFramework::Python,
        Path::new("/repo"),
        &config,
        "app/users.py",
    ));
    assert!(!is_language_native_change(
        TestFramework::Python,
        Path::new("/repo"),
        &config,
        "app/test_users.py",
    ));
}

#[test]
fn cargo_manifest_under_configured_package_is_native() {
    let mut config = NoMistakesConfig::default();
    config.tests.rust.packages = vec!["app".to_string()];
    assert!(is_language_native_change(
        TestFramework::Cargo,
        Path::new("/repo"),
        &config,
        "app/Cargo.toml",
    ));
}

#[test]
fn empty_language_roots_are_not_native() {
    let config = NoMistakesConfig::default();
    assert!(!is_language_native_change(
        TestFramework::Go,
        Path::new("/repo"),
        &config,
        "pkg/ping.go",
    ));
}

#[test]
fn owning_root_picks_longest_configured_prefix() {
    let mut config = NoMistakesConfig::default();
    config.tests.python.packages = vec!["app".to_string(), "app/users".to_string()];
    assert_eq!(
        owning_root(
            TestFramework::Python,
            Path::new("/repo"),
            &config,
            &PathBuf::from("/repo/app/users/views.py"),
        )
        .as_deref(),
        Some("app/users")
    );
}

#[test]
fn java_source_under_configured_package_is_native() {
    let mut config = NoMistakesConfig::default();
    config.tests.java.packages = vec![".".to_string()];
    assert!(is_language_native_change(
        TestFramework::Java,
        Path::new("/repo"),
        &config,
        "src/main/java/com/example/User.java",
    ));
    assert!(!is_language_native_change(
        TestFramework::Java,
        Path::new("/repo"),
        &config,
        "src/test/java/com/example/UserTest.java",
    ));
    assert!(!is_language_native_change(
        TestFramework::Java,
        Path::new("/repo"),
        &config,
        "src/test/java/com/example/Helper.java",
    ));
    assert!(is_language_native_change(
        TestFramework::Java,
        Path::new("/repo"),
        &config,
        "pom.xml",
    ));
}

#[test]
fn kotlin_source_under_configured_package_is_native() {
    let mut config = NoMistakesConfig::default();
    config.tests.kotlin.packages = vec![".".to_string()];
    assert!(is_language_native_change(
        TestFramework::Kotlin,
        Path::new("/repo"),
        &config,
        "src/main/kotlin/com/example/User.kt",
    ));
    assert!(!is_language_native_change(
        TestFramework::Kotlin,
        Path::new("/repo"),
        &config,
        "src/test/kotlin/com/example/UserTest.kt",
    ));
    assert!(!is_language_native_change(
        TestFramework::Kotlin,
        Path::new("/repo"),
        &config,
        "src/test/kotlin/com/example/Helper.kt",
    ));
    assert!(is_language_native_change(
        TestFramework::Kotlin,
        Path::new("/repo"),
        &config,
        "build.gradle.kts",
    ));
}

#[test]
fn elixir_source_under_configured_app_is_native() {
    let mut config = NoMistakesConfig::default();
    config.tests.elixir.apps = vec![".".to_string()];
    assert!(is_language_native_change(
        TestFramework::Elixir,
        Path::new("/repo"),
        &config,
        "lib/my_app/user.ex",
    ));
    assert!(!is_language_native_change(
        TestFramework::Elixir,
        Path::new("/repo"),
        &config,
        "test/my_app/user_test.exs",
    ));
    assert!(!is_language_native_change(
        TestFramework::Elixir,
        Path::new("/repo"),
        &config,
        "test/support/data_case.ex",
    ));
    assert!(is_language_native_change(
        TestFramework::Elixir,
        Path::new("/repo"),
        &config,
        "mix.exs",
    ));
    assert!(!is_language_native_change(
        TestFramework::Elixir,
        Path::new("/repo"),
        &config,
        "config/config.exs",
    ));
    assert!(!is_language_native_change(
        TestFramework::Elixir,
        Path::new("/repo"),
        &config,
        "mix.lock",
    ));
}
