use super::*;
use serde_yaml::Value;

fn locations(yaml: &str, var: &str) -> Vec<CiEnvLocation> {
    let value: Value = serde_yaml::from_str(yaml).unwrap();
    collect_locations(&value, var, &reference_regex(var))
}

#[test]
fn non_mapping_root_yields_nothing() {
    assert!(locations("- a\n- b\n", "X").is_empty());
}

#[test]
fn reference_regex_requires_standalone_env_context() {
    let re = reference_regex("FOO");
    assert!(re.is_match("echo ${{ env.FOO }}"));
    assert!(re.is_match("${{env.FOO}}"));
    // A property segment named `env` must not match.
    assert!(!re.is_match("${{ github.event.inputs.env.FOO }}"));
    assert!(!re.is_match("${{ env.OTHER }}"));
}

#[test]
fn reference_regex_matches_index_syntax() {
    let re = reference_regex("FOO");
    assert!(re.is_match("${{ env['FOO'] }}"));
    assert!(re.is_match("${{ env[\"FOO\"] }}"));
    assert!(!re.is_match("${{ env['OTHER'] }}"));
}

#[test]
fn null_job_body_is_skipped() {
    let found = locations("jobs:\n  empty:\n  real:\n    env:\n      X: y\n", "X");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].scope, EnvScope::Job);
}

#[test]
fn workflow_without_jobs_is_handled() {
    let found = locations("env:\n  X: y\n", "X");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].scope, EnvScope::Workflow);
}

#[test]
fn scalar_to_string_handles_bool_and_non_scalar() {
    assert_eq!(
        super::collect::scalar_to_string(&Value::Bool(true)).as_deref(),
        Some("true")
    );
    assert!(super::collect::scalar_to_string(&Value::Null).is_none());
}
