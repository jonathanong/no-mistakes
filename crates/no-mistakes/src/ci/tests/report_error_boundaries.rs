use super::*;

fn invalid_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/invalid-rule-path-filter/fixture/.no-mistakes.yml")
}

fn assert_invalid_glob(error: anyhow::Error) {
    assert!(
        error.to_string().contains("contains invalid glob"),
        "{error}"
    );
}

#[test]
fn report_apis_and_cli_commands_propagate_config_validation_errors() {
    let root = fixture("triggers");
    let config = invalid_config();
    assert_invalid_glob(impact_report(&root, Some(&config), &[]).unwrap_err());
    assert_invalid_glob(env_report(&root, Some(&config), "CI").unwrap_err());
    assert_invalid_glob(topology_report(&root, Some(&config), &[]).unwrap_err());

    let commands = [
        CiCommand::Impact(CiImpactArgs {
            files: Vec::new(),
            root: root.clone(),
            config: Some(config.clone()),
            format: Some(Format::Json),
            json: false,
        }),
        CiCommand::Env(CiEnvArgs {
            var: "CI".to_string(),
            root: root.clone(),
            config: Some(config.clone()),
            format: Some(Format::Json),
            json: false,
        }),
        CiCommand::Topology(CiTopologyArgs {
            workflows: Vec::new(),
            root,
            config: Some(config),
            format: Some(TopologyFormat::Json),
            json: false,
        }),
    ];
    for command in commands {
        assert_invalid_glob(run(CiArgs { command }).unwrap_err());
    }
}
