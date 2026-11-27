use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn github_actions_options_are_validated_before_analysis() {
    let root = rules_path("options-audit", "fixture", "github-actions-validation");
    let mut configs: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "yml"))
        .collect();
    configs.sort();
    let cases: Vec<_> = configs
        .iter()
        .map(|config| {
            let name = config.file_stem().unwrap().to_str().unwrap();
            (name, root.clone(), config.clone())
        })
        .collect();
    let report = assert_saved_fixture_report("github_actions_option_validation", &cases, &[], &[]);
    assert_eq!(report["valid"]["exitCode"], 0);
    for (name, _, _) in cases {
        if name == "valid" {
            continue;
        }
        let result = &report[name];
        assert_eq!(result["exitCode"], 2, "{name}: {result}");
        assert!(result["report"].is_null(), "{name}: {result}");
        let location = format!("{name}.yml:");
        assert!(
            result["stderr"].as_str().unwrap().contains(&location),
            "{result}"
        );
    }
}
