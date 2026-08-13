use super::*;

#[test]
fn input_dependent_composite_conditions_cannot_hide_static_failures() {
    let action = "name: Conditional failure\ndescription: Conditional failure\ninputs:\n  enabled: {description: Enable failure}\nruns:\n  using: composite\n  steps:\n    - if: '${{ inputs.enabled }}'\n      run: 'false'\n      shell: bash\n";
    assert!(!valid(&[("action", action)], &[], "action"));
}

#[test]
fn case_insensitive_composite_input_references_cannot_hide_static_failures() {
    let action = "name: Conditional failure\ndescription: Conditional failure\ninputs:\n  command: {description: Command}\nruns:\n  using: composite\n  steps:\n    - run: '${{ INPUTS.command }}'\n      shell: bash\n";
    assert!(!valid(&[("action", action)], &[], "action"));
}

#[test]
fn action_pre_and_post_if_conditions_require_valid_documented_contexts() {
    let node = "name: Node\ndescription: Invalid\nruns: {using: node24, pre: setup.js, pre-if: \"runner.os == 'Linux' && always()\", main: dist/index.js}";
    assert!(!valid(
        &[("action", node)],
        &["action/setup.js", "action/dist/index.js"],
        "action"
    ));
    let docker = "name: Docker\ndescription: Valid\nruns: {using: docker, image: 'docker://alpine:3.22', pre-entrypoint: setup.sh, pre-if: \"github.ref != ''\"}";
    assert!(valid(&[("action", docker)], &["action/setup.sh"], "action"));
    for pre_if in [
        "runner.os ==",
        "steps.setup.outputs.ready",
        "hashFiles('**') != ''",
    ] {
        let docker = format!("name: Docker\ndescription: Invalid\nruns: {{using: docker, image: 'docker://alpine:3.22', pre-entrypoint: setup.sh, pre-if: \"{pre_if}\"}}");
        assert!(
            !valid(&[("action", &docker)], &["action/setup.sh"], "action"),
            "{pre_if}"
        );
    }
    for post_if in [
        "runner.os ==",
        "steps.setup.outputs.ready",
        "hashFiles('**') != ''",
    ] {
        let node = format!("name: Node\ndescription: Invalid\nruns: {{using: node24, main: dist/index.js, post: cleanup.js, post-if: \"{post_if}\"}}");
        assert!(
            !valid(
                &[("action", &node)],
                &["action/dist/index.js", "action/cleanup.js"],
                "action"
            ),
            "{post_if}"
        );
    }
}

#[test]
fn composite_step_ids_are_static_unique_identifiers() {
    for steps in [
        "[{id: setup, run: echo ok, shell: bash}, {id: SETUP, run: echo ok, shell: bash}]",
        "[{id: 'setup step', run: echo ok, shell: bash}]",
        "[{id: '${{ inputs.step }}', run: echo ok, shell: bash}]",
    ] {
        let action = format!(
            "name: Invalid\ndescription: Invalid\nruns: {{using: composite, steps: {steps}}}"
        );
        assert!(!valid(&[("action", &action)], &[], "action"), "{steps}");
    }
    let valid_ids = "name: Valid\ndescription: Valid\nruns: {using: composite, steps: [{id: setup, run: echo ok, shell: bash}, {id: verify_2, run: echo ok, shell: bash}]}";
    assert!(valid(&[("action", valid_ids)], &[], "action"));
}
