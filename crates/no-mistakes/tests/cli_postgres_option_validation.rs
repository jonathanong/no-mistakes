use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_options_are_validated_before_analysis() {
    let root = rules_path("options-audit", "fixture", "postgres-validation");
    let mut configs: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter(|path| path.file_name().is_some_and(|name| name != "schema.json"))
        .collect();
    configs.sort();
    let cases: Vec<_> = configs
        .iter()
        .map(|config| {
            let name = config.file_stem().unwrap().to_str().unwrap();
            (name, root.clone(), config.clone())
        })
        .collect();
    let report = assert_saved_fixture_report("postgres_option_validation", &cases, &[], &[]);
    assert_eq!(report["valid"]["exitCode"], 0, "{}", report["valid"]);
    for (name, _, _) in cases {
        if name == "valid" {
            continue;
        }
        let result = &report[name];
        assert_eq!(result["exitCode"], 2, "{name}: {result}");
        assert!(result["report"].is_null(), "{name}: {result}");
        let location = format!("{name}.json:");
        assert!(
            result["stderr"].as_str().unwrap().contains(&location),
            "{result}"
        );
    }
}
