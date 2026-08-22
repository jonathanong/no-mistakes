use crate::codebase::test_discovery::targets::target_for;
use crate::codebase::test_discovery::TestRunner;

#[test]
fn python_pytest_target_uses_file_path() {
    let target = target_for(
        TestRunner::Python,
        Some("app"),
        false,
        None,
        "app/test_users.py",
    );
    assert_eq!(target.base_command, vec!["pytest"]);
    assert_eq!(target.runner_args, vec!["app/test_users.py"]);
    assert_eq!(target.runner, "python");
}

#[test]
fn python_tests_py_uses_unittest_module() {
    let target = target_for(TestRunner::Python, Some("app"), false, None, "app/tests.py");
    assert_eq!(target.base_command, vec!["python", "-m", "unittest"]);
    assert_eq!(target.runner_args, vec!["app.tests"]);
}

#[test]
fn go_target_is_relative_to_module() {
    let target = target_for(
        TestRunner::Go,
        Some("worker"),
        false,
        None,
        "worker/pkg/ping_test.go",
    );
    assert_eq!(target.base_command, vec!["go", "test"]);
    assert_eq!(target.runner_args, vec!["./pkg"]);
}

#[test]
fn cargo_sibling_tests_omits_test_flag() {
    let target = target_for(
        TestRunner::Cargo,
        Some("app"),
        false,
        Some("app"),
        "app/src/tests.rs",
    );
    assert_eq!(target.base_command, vec!["cargo", "test"]);
    assert_eq!(target.runner_args, vec!["-p", "app"]);
}

#[test]
fn cargo_integration_target_uses_test_flag() {
    let target = target_for(
        TestRunner::Cargo,
        Some("app"),
        false,
        Some("app"),
        "app/tests/integration.rs",
    );
    assert_eq!(
        target.runner_args,
        vec!["-p", "app", "--test", "integration"]
    );
}

#[test]
fn rails_picks_rspec_or_minitest_from_filename() {
    let spec = target_for(
        TestRunner::Rails,
        Some("."),
        false,
        None,
        "spec/jobs/welcome_job_spec.rb",
    );
    assert_eq!(spec.base_command, vec!["rspec"]);
    let minitest = target_for(
        TestRunner::Rails,
        Some("."),
        false,
        None,
        "test/models/user_test.rb",
    );
    assert_eq!(minitest.base_command, vec!["bin/rails", "test"]);
}

#[test]
fn language_target_for_rejects_js_runners() {
    for runner in [
        TestRunner::Vitest,
        TestRunner::Playwright,
        TestRunner::Jest,
        TestRunner::Dotnet,
        TestRunner::Swift,
    ] {
        assert!(
            super::language_target_for(runner, None, None, "src/value.test.ts").is_none(),
            "{runner:?} should not use a language target",
        );
    }
}

#[test]
fn php_laravel_uses_artisan_test() {
    let target = target_for(
        TestRunner::Php,
        Some("."),
        false,
        Some("laravel"),
        "tests/UserControllerTest.php",
    );
    assert_eq!(target.base_command, vec!["php", "artisan", "test"]);
    let phpunit = target_for(
        TestRunner::Php,
        Some("."),
        false,
        None,
        "tests/UserControllerTest.php",
    );
    assert_eq!(phpunit.base_command, vec!["phpunit"]);
}

#[test]
fn java_target_uses_maven_class_name() {
    let target = target_for(
        TestRunner::Java,
        Some("."),
        false,
        None,
        "src/test/java/com/example/UserTest.java",
    );
    assert_eq!(target.base_command, vec!["mvn", "test"]);
    assert_eq!(target.runner_args, vec!["-Dtest=UserTest"]);
    assert_eq!(target.runner, "java");
}

#[test]
fn kotlin_target_uses_gradle_class_name() {
    let target = target_for(
        TestRunner::Kotlin,
        Some("."),
        false,
        None,
        "src/test/kotlin/com/example/UserTest.kt",
    );
    assert_eq!(target.base_command, vec!["gradle", "test"]);
    assert_eq!(target.runner_args, vec!["--tests", "UserTest"]);
    assert_eq!(target.runner, "kotlin");
}

#[test]
fn kotlin_nested_package_passes_gradle_project_flag() {
    let target = target_for(
        TestRunner::Kotlin,
        Some("services/api"),
        false,
        None,
        "services/api/src/test/kotlin/com/example/UserTest.kt",
    );
    assert_eq!(
        target.base_command,
        vec!["gradle", "-p", "services/api", "test"]
    );
    assert_eq!(target.runner_args, vec!["--tests", "UserTest"]);
}

#[test]
fn elixir_target_uses_mix_test() {
    let target = target_for(
        TestRunner::Elixir,
        Some("."),
        false,
        None,
        "test/my_app/user_test.exs",
    );
    assert_eq!(target.base_command, vec!["mix", "test"]);
    assert_eq!(target.runner_args, vec!["test/my_app/user_test.exs"]);
    assert_eq!(target.runner, "elixir");
}

#[test]
fn elixir_nested_app_passes_mix_change_directory_flag() {
    let target = target_for(
        TestRunner::Elixir,
        Some("apps/web"),
        false,
        None,
        "apps/web/test/my_app/user_test.exs",
    );
    assert_eq!(
        target.base_command,
        vec!["mix", "-C", "apps/web", "test"]
    );
    assert_eq!(target.runner_args, vec!["test/my_app/user_test.exs"]);
}

#[test]
fn java_nested_package_passes_maven_file_flag() {
    let target = target_for(
        TestRunner::Java,
        Some("services/api"),
        false,
        None,
        "services/api/src/test/java/com/example/UserTest.java",
    );
    assert_eq!(
        target.runner_args,
        vec!["-f", "services/api/pom.xml", "-Dtest=UserTest"]
    );
}
