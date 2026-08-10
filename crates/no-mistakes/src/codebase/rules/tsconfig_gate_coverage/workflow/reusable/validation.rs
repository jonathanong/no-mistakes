use serde_yaml::Value;

pub(super) fn canonical_local_call_target(target: &str) -> bool {
    target
        .strip_prefix("./.github/workflows/")
        .is_some_and(|filename| {
            !filename.is_empty()
                && !filename.contains(['/', '\\'])
                && (filename.ends_with(".yml") || filename.ends_with(".yaml"))
        })
}

pub(super) fn reusable_call_job_shape_valid(job: &Value) -> bool {
    job.as_mapping().is_some_and(|mapping| {
        mapping.keys().all(|key| {
            key.as_str().is_some_and(|key| {
                matches!(
                    key,
                    "name"
                        | "uses"
                        | "with"
                        | "secrets"
                        | "strategy"
                        | "needs"
                        | "if"
                        | "concurrency"
                        | "permissions"
                )
            })
        })
    })
}

pub(super) fn workflow_call_shape_valid(on: Option<&Value>) -> bool {
    let Some(contract) = on.and_then(|on| on.get("workflow_call")) else {
        return true;
    };
    let Value::Mapping(contract) = contract else {
        return matches!(contract, Value::Null);
    };
    ["inputs", "secrets", "outputs"].into_iter().all(|field| {
        contract.get(field).is_none_or(|declarations| {
            declarations
                .as_mapping()
                .is_some_and(|mapping| mapping.values().all(|declaration| declaration.is_mapping()))
        })
    })
}

pub(super) fn zero_instance_matrix(job: &Value) -> bool {
    let Some(matrix) = job
        .get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
        .and_then(Value::as_mapping)
    else {
        return false;
    };
    let empty_axis = matrix.iter().any(|(name, values)| {
        !matches!(name.as_str(), Some("include" | "exclude"))
            && values.as_sequence().is_some_and(Vec::is_empty)
    });
    let includes_instance = match matrix.get("include") {
        Some(Value::Sequence(items)) => !items.is_empty(),
        Some(_) => return false,
        None => false,
    };
    empty_axis && !includes_instance
}
