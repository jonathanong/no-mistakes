fn java_target_for(package: Option<&str>, test_file: &str) -> TestExecutionTarget {
    let class_name = slash(test_file)
        .rsplit('/')
        .next()
        .unwrap_or(test_file)
        .strip_suffix(".java")
        .unwrap_or(test_file)
        .to_string();
    let pom = package
        .map(slash)
        .filter(|value| !value.is_empty() && value != ".");
    let mut runner_args = pom
        .map(|package| vec!["-f".into(), format!("{package}/pom.xml")])
        .unwrap_or_default();
    runner_args.push(format!("-Dtest={class_name}"));
    language_target(
        TestRunner::Java,
        package,
        None,
        vec!["mvn".to_string(), "test".to_string()],
        runner_args,
    )
}

fn kotlin_target_for(package: Option<&str>, test_file: &str) -> TestExecutionTarget {
    let class_name = slash(test_file)
        .rsplit('/')
        .next()
        .unwrap_or(test_file)
        .strip_suffix(".kt")
        .unwrap_or(test_file)
        .to_string();
    let project = package
        .map(slash)
        .filter(|value| !value.is_empty() && value != ".");
    let mut base_command = vec!["gradle".to_string()];
    if let Some(package) = project {
        base_command.push("-p".into());
        base_command.push(package);
    }
    base_command.push("test".into());
    language_target(
        TestRunner::Kotlin,
        package,
        None,
        base_command,
        vec!["--tests".into(), class_name],
    )
}
