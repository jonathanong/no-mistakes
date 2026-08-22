use super::targets::TestExecutionTarget;
use super::types::TestRunner;

pub(super) fn language_target_for(
    runner: TestRunner,
    config: Option<&str>,
    project: Option<&str>,
    test_file: &str,
) -> Option<TestExecutionTarget> {
    Some(match runner {
        TestRunner::Python => python_target_for(config, test_file),
        TestRunner::Go => go_target_for(config, test_file),
        TestRunner::Cargo => cargo_target_for(config, project, test_file),
        TestRunner::Rails => rails_target_for(config, test_file),
        TestRunner::Php => php_target_for(config, project, test_file),
        TestRunner::Java => java_target_for(config, test_file),
        TestRunner::Kotlin => kotlin_target_for(config, test_file),
        TestRunner::Dotnet
        | TestRunner::Playwright
        | TestRunner::Vitest
        | TestRunner::Jest
        | TestRunner::Swift => {
            return None;
        }
    })
}

fn python_target_for(package: Option<&str>, test_file: &str) -> TestExecutionTarget {
    let (base_command, runner_args) = if uses_unittest(test_file) {
        (
            vec![
                "python".to_string(),
                "-m".to_string(),
                "unittest".to_string(),
            ],
            vec![unittest_module(test_file)],
        )
    } else {
        (vec!["pytest".to_string()], vec![test_file.to_string()])
    };
    language_target(TestRunner::Python, package, None, base_command, runner_args)
}

fn go_target_for(module: Option<&str>, test_file: &str) -> TestExecutionTarget {
    let package_dir = package_dir_from_test(test_file);
    let rel = relative_to_config(module, &package_dir);
    language_target(
        TestRunner::Go,
        module,
        None,
        vec!["go".to_string(), "test".to_string()],
        vec![format!("./{rel}")],
    )
}

fn cargo_target_for(
    package_path: Option<&str>,
    crate_name: Option<&str>,
    test_file: &str,
) -> TestExecutionTarget {
    let pkg = crate_name
        .or(package_path)
        .unwrap_or("app")
        .trim_end_matches('/');
    let mut runner_args = vec!["-p".to_string(), pkg.to_string()];
    if let Some(name) = cargo_integration_name(test_file) {
        runner_args.push("--test".to_string());
        runner_args.push(name);
    }
    language_target(
        TestRunner::Cargo,
        package_path,
        crate_name,
        vec!["cargo".to_string(), "test".to_string()],
        runner_args,
    )
}

fn rails_target_for(app: Option<&str>, test_file: &str) -> TestExecutionTarget {
    let (base_command, runner_args) = if test_file.ends_with("_spec.rb") {
        (vec!["rspec".to_string()], vec![test_file.to_string()])
    } else {
        (
            vec!["bin/rails".to_string(), "test".to_string()],
            vec![test_file.to_string()],
        )
    };
    language_target(TestRunner::Rails, app, None, base_command, runner_args)
}

fn php_target_for(
    app: Option<&str>,
    framework: Option<&str>,
    test_file: &str,
) -> TestExecutionTarget {
    let (base_command, runner_args) = if framework == Some("laravel") {
        (
            vec!["php".to_string(), "artisan".to_string(), "test".to_string()],
            vec![test_file.to_string()],
        )
    } else {
        (vec!["phpunit".to_string()], vec![test_file.to_string()])
    };
    language_target(TestRunner::Php, app, framework, base_command, runner_args)
}

include!("lang_targets_jvm.rs");

fn language_target(
    runner: TestRunner,
    config: Option<&str>,
    project: Option<&str>,
    base_command: Vec<String>,
    runner_args: Vec<String>,
) -> TestExecutionTarget {
    TestExecutionTarget {
        runner: runner.as_str().to_string(),
        config: config.map(str::to_string),
        workspace: false,
        project: project.map(str::to_string),
        base_command,
        runner_args,
    }
}

fn uses_unittest(test_file: &str) -> bool {
    slash(test_file)
        .rsplit('/')
        .next()
        .is_some_and(|name| name == "tests.py")
}

fn unittest_module(test_file: &str) -> String {
    let path = slash(test_file);
    let without_ext = path.strip_suffix(".py").unwrap_or(&path);
    without_ext.replace('/', ".")
}

fn package_dir_from_test(test_file: &str) -> String {
    slash(test_file)
        .rsplit_once('/')
        .map(|(dir, _)| dir.to_string())
        .unwrap_or_else(|| ".".to_string())
}

fn relative_to_config(config: Option<&str>, path: &str) -> String {
    let Some(config) = config
        .map(slash)
        .filter(|value| !value.is_empty() && value != ".")
    else {
        return path.trim_start_matches("./").to_string();
    };
    let prefix = format!("{config}/");
    path.strip_prefix(&prefix)
        .unwrap_or(path)
        .trim_start_matches("./")
        .to_string()
}

fn cargo_integration_name(test_file: &str) -> Option<String> {
    let path = slash(test_file);
    let file_name = path.rsplit('/').next()?;
    let stem = file_name.strip_suffix(".rs")?;
    path.contains("/tests/")
        .then(|| stem.to_string())
        .filter(|stem| !stem.is_empty())
}

fn slash(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
#[path = "lang_targets/tests.rs"]
mod tests;
