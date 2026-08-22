fn dart_target_for(package: Option<&str>, test_file: &str) -> TestExecutionTarget {
    language_target(
        TestRunner::Dart,
        package,
        None,
        vec!["dart".to_string(), "test".to_string()],
        vec![relative_to_config(package, test_file)],
    )
}
