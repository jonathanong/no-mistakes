use super::super::options::applications;
use crate::config::v2::schema::NoMistakesConfig;
use serde_yaml::Value;
use std::path::PathBuf;

fn loaded() -> (PathBuf, NoMistakesConfig) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/errors");
    let config =
        crate::config::v2::load_v2_config_with_path(&root, Some(&root.join("broken-target.yml")))
            .unwrap()
            .0;
    (root, config)
}

#[test]
fn rejects_bad_effort_unanalyzable_origin_and_include_glob() {
    let (root, mut config) = loaded();
    config.rules[0].options["effort"] = Value::String("nope".into());
    let effort = applications(&root, &config, &[]).err().expect("bad effort");
    assert!(
        effort.to_string().contains("invalid options.effort"),
        "{effort}"
    );

    let (root, mut config) = loaded();
    config.rules[0].options["unanalyzable"] = Value::String("nope".into());
    let mode = applications(&root, &config, &[])
        .err()
        .expect("bad unanalyzable");
    assert!(
        mode.to_string().contains("invalid options.unanalyzable"),
        "{mode}"
    );

    let (root, mut config) = loaded();
    config.rules[0].options["selfOrigins"] =
        Value::Sequence(vec![Value::String("not-an-origin".into())]);
    let origin = applications(&root, &config, &[]).err().expect("bad origin");
    assert!(
        origin.to_string().contains("must be a valid origin"),
        "{origin}"
    );

    let (root, mut config) = loaded();
    config.rules[0].include = vec!["[".into()];
    let glob = applications(&root, &config, &[]).err().expect("bad glob");
    assert!(glob.to_string().contains("invalid glob"), "{glob}");
}

#[test]
fn dedups_repeated_csp_file_paths() {
    let (root, mut config) = loaded();
    let path = "../headers-text/public/_headers";
    config.rules[0].options["cspFiles"] =
        Value::Sequence(vec![Value::String(path.into()), Value::String(path.into())]);
    let apps = applications(&root, &config, &[]).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].csp_files.len(), 1);
    assert!(apps[0].csp_files[0].ends_with("_headers"));
}
