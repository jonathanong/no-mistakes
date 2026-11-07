use super::*;

mod binding_control_flow;
mod executor_selection;
mod helper_completion;
mod helper_projection;
mod helper_shapes;
mod helpers;
mod phase_two_sql;
mod placeholder_provenance;
mod proof;
mod sql_forms;

fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    check_with_files(root, config, &[root.join("src/query.ts")])
}
fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-lock-written-rows-only/fixture")
            .join(name),
    )
}
fn config() -> NoMistakesConfig {
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/fail.yml"
    )))
    .unwrap();
    config.rules[0].options["checks"] =
        serde_yaml::to_value(BTreeMap::from(CHECKS.map(|check| (check, "fail")))).unwrap();
    config
}
#[test]
fn reports_each_phase_one_lock_policy() {
    for (case, target) in [
        ("row-unwritten", CHECKS[0]),
        ("row-key-mismatch", CHECKS[0]),
        ("table-lock", CHECKS[1]),
        ("constant-advisory", CHECKS[2]),
        ("outside-transaction", CHECKS[3]),
    ] {
        let findings = check(&fixture(case), &config()).unwrap();
        assert!(
            findings
                .iter()
                .any(|finding| finding.target.as_deref() == Some(target)),
            "{case}: {findings:?}"
        );
    }
}
#[test]
fn accepts_written_rows_and_keyed_advisory_locks() {
    for case in ["row-written", "keyed-advisory"] {
        assert!(
            check(&fixture(case), &config()).unwrap().is_empty(),
            "{case}"
        );
    }
}
#[test]
fn validates_accounting_options() {
    for yaml in [
        "allow: [{relation: items}]",
        "allow: [{reason: empty}]",
        "writeSets: [{file: src.ts}]",
        "checks: {unknown: fail}",
    ] {
        let options: Options = serde_yaml::from_str(yaml).unwrap();
        assert!(compile(&options).is_err(), "{yaml}");
    }
}
