fn dart_target_for(package: Option<&str>, test_file: &str) -> TestExecutionTarget {
    let nested = package
        .map(slash)
        .filter(|value| !value.is_empty() && value != ".");
    if let Some(dir) = nested {
        return language_target(
            TestRunner::Dart,
            package,
            None,
            vec![
                "dart".into(),
                "pub".into(),
                "--directory".into(),
                dir,
                "run".into(),
                "test".into(),
            ],
            vec![relative_to_config(package, test_file)],
        );
    }
    language_target(
        TestRunner::Dart,
        package,
        None,
        vec!["dart".to_string(), "test".to_string()],
        vec![slash(test_file).trim_start_matches("./").to_string()],
    )
}
