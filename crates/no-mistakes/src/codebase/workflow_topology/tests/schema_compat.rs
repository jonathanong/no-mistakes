pub fn assert_legacy_golden(actual: &str, expected: &str) {
    assert_eq!(
        legacy_projection(super::json(actual)),
        super::json(expected)
    );
}

pub fn legacy_projection(mut value: serde_json::Value) -> serde_json::Value {
    if let Some(workflows) = value
        .get_mut("workflows")
        .and_then(serde_json::Value::as_array_mut)
    {
        for workflow in workflows {
            if let Some(workflow) = workflow.as_object_mut() {
                workflow.remove("env");
                workflow.remove("secretReferences");
            }
        }
    }
    if let Some(jobs) = value
        .get_mut("jobs")
        .and_then(serde_json::Value::as_array_mut)
    {
        for job in jobs {
            strip_step_fields(job);
            if let Some(job) = job.as_object_mut() {
                for field in [
                    "environment",
                    "timeoutMinutes",
                    "runsOn",
                    "permissions",
                    "outputs",
                    "env",
                    "secretReferences",
                ] {
                    job.remove(field);
                }
            }
        }
    }
    value
}

fn strip_step_fields(job: &mut serde_json::Value) {
    let Some(steps) = job
        .get_mut("steps")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };
    for step in steps {
        if let Some(step) = step.as_object_mut() {
            for field in ["run", "with", "env", "secretReferences"] {
                step.remove(field);
            }
        }
    }
}
