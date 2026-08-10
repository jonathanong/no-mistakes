use super::*;

fn jobs(yaml: &str) -> serde_yaml::Mapping {
    serde_yaml::from_str::<Value>(yaml)
        .unwrap()
        .get("jobs")
        .and_then(Value::as_mapping)
        .unwrap()
        .clone()
}

#[test]
fn dependency_validation_rejects_malformed_and_unresolvable_jobs() {
    assert!(!valid_job_dependencies(&jobs("jobs:\n  ? [bad]\n  : {}")));
    assert!(!valid_job_dependencies(&jobs("jobs:\n  bad: []")));
    assert!(!valid_job_dependencies(&jobs(
        "jobs:\n  bad:\n    needs: 1"
    )));
    assert!(!valid_job_dependencies(&jobs(
        "jobs:\n  bad:\n    needs: [valid, 1]\n  valid: {}"
    )));
    assert!(!valid_job_dependencies(&jobs("jobs:\n  A: {}\n  a: {}")));
    for job_id in ["1build", "build job", "$x"] {
        assert!(!valid_job_dependencies(&jobs(&format!(
            "jobs:\n  '{job_id}': {{}}"
        ))));
    }
    assert!(!valid_job_dependencies(&jobs(
        "jobs:\n  bad:\n    needs: missing"
    )));
}
