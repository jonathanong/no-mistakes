use super::*;
use crate::codebase::workflow_topology::model::WorkflowCallSecret;

fn contract(name: &str) -> WorkflowCallContract {
    WorkflowCallContract {
        secrets: BTreeMap::from([(
            name.to_string(),
            WorkflowCallSecret {
                required: true,
                description: None,
            },
        )]),
        ..WorkflowCallContract::default()
    }
}

#[test]
fn inherited_secrets_preserve_wildcard_or_known_availability() {
    let inherit: Value = serde_yaml::from_str("secrets: inherit").unwrap();
    assert!(callee_secrets(&contract("token"), &inherit, &SecretState::direct()).is_some());

    let explicit: Value =
        serde_yaml::from_str("secrets:\n  token: '${{ secrets.TOKEN }}'").unwrap();
    let available = callee_secrets(&contract("token"), &explicit, &SecretState::direct()).unwrap();
    assert!(callee_secrets(&contract("token"), &inherit, &available).is_some());
    assert!(callee_secrets(&contract("other"), &inherit, &available).is_none());
}

#[test]
fn valid_secret_binding_contexts_supply_the_destination_secret() {
    for binding in [
        "${{ secrets['TOKEN'] }}",
        "${{ github.token }}",
        "${{ needs.setup.outputs.token }}",
        "${{ strategy.job-index }}",
        "${{ matrix.token }}",
        "${{ inputs.token }}",
        "${{ vars.TOKEN }}",
        "${{ secrets.MISSING || github.token }}",
    ] {
        let call_job: Value =
            serde_yaml::from_str(&format!("secrets:\n  token: \"{binding}\"")).unwrap();
        assert!(
            callee_secrets(&contract("token"), &call_job, &SecretState::direct()).is_some(),
            "{binding}"
        );
    }
}

#[test]
fn unavailable_secret_binding_contexts_do_not_supply_the_destination_secret() {
    for binding in [
        "${{ steps.setup.outputs.token }}",
        "${{ env.TOKEN }}",
        "${{ runner.os }}",
    ] {
        let call_job: Value =
            serde_yaml::from_str(&format!("secrets:\n  token: \"{binding}\"")).unwrap();
        assert!(
            callee_secrets(&contract("token"), &call_job, &SecretState::direct()).is_none(),
            "{binding}"
        );
    }
}
