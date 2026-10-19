use super::{args, generate_impacted_checks_with_stats, multi_framework_fixture};

#[test]
fn configured_swift_argv_keeps_generated_package_filter() {
    let root = multi_framework_fixture();
    let mut arguments = args(&["swift/App/Sources/App/Value.swift"]);
    arguments.root = root.clone();
    arguments.config = Some(root.join("swift-runner.no-mistakes.yml"));

    let (report, _) = generate_impacted_checks_with_stats(&arguments).unwrap();
    let swift = report
        .checks
        .iter()
        .find(|check| check.name == "swift")
        .expect("swift check");
    assert_eq!(
        swift.command,
        vec![
            "host's swift".to_string(),
            "test".to_string(),
            "--package-path".to_string(),
            "swift/App".to_string(),
            "--filter".to_string(),
            "AppTests".to_string(),
        ]
    );
}
