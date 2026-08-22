use super::*;

fn lang_test_plan_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

#[test]
fn language_frameworks_emit_native_test_commands() {
    // Reuse the existing tests-plan fixtures so impacted-checks language
    // coverage stays on the same configured packages as `tests plan`.
    let cases: &[(&str, &[&str], &str)] = &[
        ("python-test-plan", &["app/users.py"], "pytest"),
        ("go-test-plan", &["pkg/ping.go"], "go test"),
        ("cargo-test-plan", &["app/src/lib.rs"], "cargo test -p app"),
        ("rails-test-plan", &["app/models/user.rb"], "bin/rails test"),
        (
            "php-test-plan",
            &["app/Http/Controllers/UserController.php"],
            "php artisan test",
        ),
        (
            "java-test-plan",
            &["src/main/java/com/example/User.java"],
            "mvn test",
        ),
        (
            "kotlin-test-plan",
            &["src/main/kotlin/com/example/User.kt"],
            "gradle test",
        ),
    ];
    for (name, files, needle) in cases {
        let mut a = args(files);
        a.root = lang_test_plan_fixture(name);
        let (report, stats) = generate_impacted_checks_with_stats(&a).unwrap();
        let commands = command_strings(&report);
        assert!(
            commands.iter().any(|cmd| cmd.contains(needle)),
            "{name}: expected `{needle}` in {commands:?}"
        );
        assert!(
            stats.framework_discoveries >= 1,
            "{name}: expected a language framework discovery"
        );
        assert!(report.warnings.is_empty(), "{name}: {:?}", report.warnings);
    }
}

#[test]
fn framework_present_detects_configured_language_packages() {
    let cases = [
        ("python-test-plan", TestFramework::Python, TestFramework::Go),
        ("go-test-plan", TestFramework::Go, TestFramework::Python),
        (
            "cargo-test-plan",
            TestFramework::Cargo,
            TestFramework::Rails,
        ),
        ("rails-test-plan", TestFramework::Rails, TestFramework::Php),
        ("php-test-plan", TestFramework::Php, TestFramework::Cargo),
        ("java-test-plan", TestFramework::Java, TestFramework::Php),
        (
            "kotlin-test-plan",
            TestFramework::Kotlin,
            TestFramework::Php,
        ),
    ];
    for (name, present, absent) in cases {
        let root = lang_test_plan_fixture(name);
        let config = crate::config::v2::load_v2_config(&root, None)
            .expect("language test-plan fixtures ship a config");
        let visible = crate::codebase::ts_source::discover_visible_paths(&root);
        assert!(
            framework_present(&root, &config, present, &visible),
            "{name}: {present:?} should be present"
        );
        assert!(
            !framework_present(&root, &config, absent, &visible),
            "{name}: {absent:?} should be absent"
        );
    }
}
