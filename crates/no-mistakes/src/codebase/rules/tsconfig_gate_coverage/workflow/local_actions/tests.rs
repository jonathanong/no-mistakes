use super::*;
use crate::codebase::ci_workflows::{ParsedWorkflowDocument, ParsedWorkflowSet};
use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;
use serde_yaml::Value;

impl LocalActionCatalog {
    pub(crate) fn non_docker(actions: BTreeSet<String>) -> Self {
        Self(
            actions
                .into_iter()
                .map(|action| (action, LocalActionKind::Other))
                .collect(),
        )
    }

    pub(crate) fn docker(actions: BTreeSet<String>) -> Self {
        Self(
            actions
                .into_iter()
                .map(|action| (action, LocalActionKind::Docker))
                .collect(),
        )
    }
}

fn descriptors(entries: &[(&str, &str)]) -> BTreeMap<String, Value> {
    entries
        .iter()
        .map(|(directory, yaml)| {
            (
                (*directory).to_string(),
                serde_yaml::from_str(yaml).expect("valid fixture YAML"),
            )
        })
        .collect()
}

fn valid(entries: &[(&str, &str)], tracked: &[&str], directory: &str) -> bool {
    action_directory_valid(
        directory,
        &descriptors(entries),
        &tracked.iter().map(|path| (*path).to_string()).collect(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    )
}

#[test]
fn action_metadata_requires_a_supported_complete_execution_contract() {
    for yaml in [
        "name: Composite\ndescription: Valid\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Docker\ndescription: Valid\nruns: {using: docker, image: Dockerfile}",
        "name: Local build target\ndescription: Valid\nruns: {using: docker, image: build/container}",
        "name: Container\ndescription: Valid\nruns: {using: docker, image: 'docker://alpine:3.22'}",
        "name: Upper container\ndescription: Valid\nruns: {using: docker, image: 'DOCKER://alpine:3.22'}",
        "name: Node 20\ndescription: Valid\nruns: {using: node20, main: dist/index.js}",
        "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js}",
    ] {
        let tracked = if yaml.contains("using: node") {
            &["action/dist/index.js"][..]
        } else if yaml.contains("image: Dockerfile") {
            &["action/Dockerfile"][..]
        } else if yaml.contains("image: build/container") {
            &["action/build/container"][..]
        } else {
            &[]
        };
        assert!(valid(&[("action", yaml)], tracked, "action"), "{yaml}");
    }

    for yaml in [
        "[]",
        "description: Missing name\nruns: {using: node20, main: index.js}",
        "name: Blank description\ndescription: '  '\nruns: {using: node20, main: index.js}",
        "name: Missing runs\ndescription: Invalid",
        "name: Empty composite\ndescription: Invalid\nruns: {using: composite, steps: []}",
        "name: Empty step\ndescription: Invalid\nruns: {using: composite, steps: [{}]}",
        "name: Missing shell\ndescription: Invalid\nruns: {using: composite, steps: [{run: ok}]}",
        "name: Unsupported step field\ndescription: Invalid\nruns: {using: composite, steps: [{run: ok, shell: bash, timeout-minutes: 1}]}",
        "name: Empty image\ndescription: Invalid\nruns: {using: docker, image: ''}",
        "name: Non-string image\ndescription: Invalid\nruns: {using: docker, image: [Dockerfile]}",
        "name: Padded Dockerfile\ndescription: Invalid\nruns: {using: docker, image: ' Dockerfile '}",
        "name: Empty container\ndescription: Invalid\nruns: {using: docker, image: 'docker://'}",
        "name: Malformed container\ndescription: Invalid\nruns: {using: docker, image: 'docker://ghcr.io//checker:22'}",
        "name: Incomplete container tag\ndescription: Invalid\nruns: {using: docker, image: 'docker://node:'}",
        "name: Invalid container\ndescription: Invalid\nruns: {using: docker, image: 'docker://bad image'}",
        "name: Bare container image\ndescription: Invalid\nruns: {using: docker, image: node:20}",
        "name: Missing Dockerfile\ndescription: Invalid\nruns: {using: docker, image: Dockerfile}",
        "name: Empty main\ndescription: Invalid\nruns: {using: node20, main: ''}",
        "name: Future runtime\ndescription: Invalid\nruns: {using: node99, main: index.js}",
    ] {
        assert!(!valid(&[("action", yaml)], &[], "action"), "{yaml}");
    }

    let node = "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js}";
    assert!(!valid(&[("action", node)], &[], "action"));
    assert!(!valid(
        &[("action", node)],
        &["outside/dist/index.js"],
        "action"
    ));
    let escaping = "name: Node\ndescription: Invalid\nruns: {using: node24, main: ../outside.js}";
    assert!(!valid(&[("action", escaping)], &["outside.js"], "action"));
    let absolute = "name: Node\ndescription: Invalid\nruns: {using: node24, main: /outside.js}";
    assert!(!valid(
        &[("action", absolute)],
        &["action/outside.js"],
        "action"
    ));

    let root_node =
        "name: Root node\ndescription: Valid\nruns: {using: node24, main: dist/index.js}";
    assert!(valid(&[("", root_node)], &["dist/index.js"], ""));
    let root_docker =
        "name: Root Docker\ndescription: Valid\nruns: {using: docker, image: Dockerfile}";
    assert!(valid(&[("", root_docker)], &["Dockerfile"], ""));
    for runtime in ["node12", "node16"] {
        let legacy = format!(
            "name: Legacy\ndescription: Invalid\nruns: {{using: {runtime}, main: dist/index.js}}"
        );
        assert!(!valid(
            &[("action", &legacy)],
            &["action/dist/index.js"],
            "action"
        ));
    }
    let local_node_hooks = "name: Node\ndescription: Invalid\nruns: {using: node24, pre: missing-pre.js, main: dist/index.js, post: cleanup.js}";
    assert!(!valid(
        &[("action", local_node_hooks)],
        &["action/dist/index.js"],
        "action"
    ));
    let local_node_post_hook = "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js, post: cleanup.js, post-if: always()}";
    assert!(!valid(
        &[("action", local_node_post_hook)],
        &["action/dist/index.js"],
        "action"
    ));
    assert!(valid(
        &[("action", local_node_post_hook)],
        &["action/dist/index.js", "action/cleanup.js"],
        "action"
    ));
    let local_node_pre_hook = "name: Node\ndescription: Invalid\nruns: {using: node24, pre: setup.js, pre-if: always(), main: dist/index.js}";
    assert!(!valid(
        &[("action", local_node_pre_hook)],
        &["action/setup.js", "action/dist/index.js"],
        "action"
    ));
    let escaping_dockerfile =
        "name: Docker\ndescription: Invalid\nruns: {using: docker, image: ../Dockerfile}";
    assert!(!valid(
        &[("action", escaping_dockerfile)],
        &["Dockerfile"],
        "action"
    ));
    let suffixed_dockerfile =
        "name: Docker\ndescription: Valid\nruns: {using: docker, image: build/Dockerfile.test}";
    assert!(valid(
        &[("action", suffixed_dockerfile)],
        &["action/build/Dockerfile.test"],
        "action"
    ));
}

#[test]
fn action_metadata_coerces_scalar_defaults_and_rejects_unavailable_composite_contexts() {
    let defaults = "name: Defaults\ndescription: Valid\ninputs:\n  enabled: {description: Enabled, default: true}\n  retries: {description: Retries, default: 3}\nruns: {using: node24, main: dist/index.js}";
    assert!(valid(
        &[("action", defaults)],
        &["action/dist/index.js"],
        "action"
    ));

    for step in [
        "{env: {TOKEN: '${{ secrets.TOKEN }}'}, run: echo ok, shell: bash}",
        "{run: 'echo ${{ secrets.TOKEN }}', shell: bash}",
    ] {
        let action = format!(
            "name: Invalid\ndescription: Invalid\nruns: {{using: composite, steps: [{step}]}}"
        );
        assert!(!valid(&[("action", &action)], &[], "action"), "{step}");
    }
}

#[test]
fn action_metadata_validates_all_supported_fields_before_cataloging() {
    let complete = "name: Complete\nauthor: Acme\ndescription: Valid\ninputs:\n  project:\n    description: Project name\n    required: true\n    default: app\n    deprecationMessage: Use matrix.project\noutputs:\n  result:\n    description: Result path\n    value: '${{ steps.result.outputs.path }}'\nruns:\n  using: composite\n  steps:\n    - id: result\n      run: echo ok\n      shell: bash\nbranding:\n  icon: check\n  color: green\n";
    assert!(valid(&[("action", complete)], &[], "action"));
    for icon in ["archive", "arrow-down-circle", "x", "zap-off", "zoom-out"] {
        let boundary_icon = format!(
            "name: Boundary\ndescription: Valid\nbranding: {{icon: {icon}, color: blue}}\nruns: {{using: composite, steps: [{{run: ok, shell: bash}}]}}"
        );
        assert!(
            valid(&[("action", &boundary_icon)], &[], "action"),
            "{icon}"
        );
    }
    for icon in ["not-a-feather-icon", "coffee"] {
        let invalid_icon = format!(
            "name: Invalid\ndescription: Invalid\nbranding: {{icon: {icon}, color: green}}\nruns: {{using: composite, steps: [{{run: ok, shell: bash}}]}}"
        );
        assert!(
            !valid(&[("action", &invalid_icon)], &[], "action"),
            "{icon}"
        );
    }
    let tolerated = "name: Tolerated\ndescription: Valid\nruns:\n  using: composite\n  steps:\n    - run: 'false'\n      shell: bash\n      continue-on-error: true\n";
    assert!(valid(&[("action", tolerated)], &[], "action"));

    for default in ["app", "true", "42", "null"] {
        let action = format!(
            "name: Scalar default\ndescription: Valid\ninputs: {{project: {{description: Project, default: {default}}}}}\nruns: {{using: composite, steps: [{{run: echo ok, shell: bash}}]}}"
        );
        assert!(valid(&[("action", &action)], &[], "action"), "{default}");
    }

    for yaml in [
        "name: Unknown top-level\ndescription: Invalid\nunknown: true\nruns: {using: node24, main: index.js}",
        "name: Bad author\nauthor: [Acme]\ndescription: Invalid\nruns: {using: node24, main: index.js}",
        "name: Bad inputs\ndescription: Invalid\ninputs: []\nruns: {using: node24, main: index.js}",
        "name: Non-string input name\ndescription: Invalid\ninputs: {1: {description: Project}}\nruns: {using: node24, main: index.js}",
        "name: Non-mapping input metadata\ndescription: Invalid\ninputs: {project: Project}\nruns: {using: node24, main: index.js}",
        "name: Missing input description\ndescription: Invalid\ninputs: {project: {required: true}}\nruns: {using: node24, main: index.js}",
        "name: Bad input required\ndescription: Invalid\ninputs: {project: {description: Project, required: yes}}\nruns: {using: node24, main: index.js}",
        "name: Bad input default\ndescription: Invalid\ninputs: {project: {description: Project, default: [app]}}\nruns: {using: node24, main: index.js}",
        "name: Bad deprecation\ndescription: Invalid\ninputs: {project: {description: Project, deprecationMessage: [old]}}\nruns: {using: node24, main: index.js}",
        "name: Bad outputs\ndescription: Invalid\noutputs: []\nruns: {using: node24, main: index.js}",
        "name: Non-string output name\ndescription: Invalid\noutputs: {1: {description: Result}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Non-mapping output metadata\ndescription: Invalid\noutputs: {result: Result}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: JavaScript output value\ndescription: Invalid\noutputs: {result: {description: Result, value: value}}\nruns: {using: node24, main: index.js}",
        "name: Missing output description\ndescription: Invalid\noutputs: {result: {value: path}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Missing composite output value\ndescription: Invalid\noutputs: {result: {description: Result}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Bad output field\ndescription: Invalid\noutputs: {result: {description: Result, unknown: true, value: path}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Bad output expression\ndescription: Invalid\noutputs: {result: {description: Result, value: '${{ steps.build.outputs. }}'}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Unavailable output context\ndescription: Invalid\noutputs: {result: {description: Result, value: '${{ needs.build.outputs.result }}'}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Bad branding\ndescription: Invalid\nbranding: {icon: check, color: pink}\nruns: {using: node24, main: index.js}",
        "name: Unknown branding icon\ndescription: Invalid\nbranding: {icon: not-a-feather-icon, color: green}\nruns: {using: node24, main: index.js}",
        "name: Omitted branding icon\ndescription: Invalid\nbranding: {icon: coffee, color: green}\nruns: {using: node24, main: index.js}",
        "name: Bad branding shape\ndescription: Invalid\nbranding: check\nruns: {using: node24, main: index.js}",
        "name: Unknown runs field\ndescription: Invalid\nruns: {using: node24, main: index.js, unknown: true}",
        "name: Unsupported local pre\ndescription: Invalid\nruns: {using: node24, pre: setup.js, main: index.js}",
        "name: Unsupported local pre condition\ndescription: Invalid\nruns: {using: node24, main: index.js, pre-if: always()}",
        "name: Bad node hook\ndescription: Invalid\nruns: {using: node24, main: index.js, post: [cleanup]}",
        "name: Bad docker args\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, args: [--ok, 1]}",
        "name: Docker args not sequence\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, args: --ok}",
        "name: Malformed Docker arg expression\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, args: ['${{ inputs. }}']}",
        "name: Unavailable Docker arg context\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, args: ['${{ steps.build.outcome }}']}",
        "name: Bad docker env\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, env: [KEY]}",
        "name: Docker env non-string value\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, env: {KEY: true}}",
        "name: Docker env non-string key\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, env: {1: value}}",
        "name: Bad composite step\ndescription: Invalid\nruns: {using: composite, steps: [true]}",
    ] {
        assert!(!valid(&[("action", yaml)], &[], "action"), "{yaml}");
    }
}

#[test]
fn composite_step_contexts_exclude_workflow_only_secrets() {
    for field in [
        "if: '${{ secrets.TOKEN != '' }}'\n      run: echo ok\n      shell: bash",
        "run: 'echo ${{ secrets.TOKEN }}'\n      shell: bash",
        "run: echo ok\n      shell: '${{ secrets.SHELL }}'",
        "run: echo ok\n      shell: bash\n      working-directory: '${{ secrets.DIRECTORY }}'",
        "run: echo ok\n      shell: bash\n      env: {TOKEN: '${{ secrets.TOKEN }}'}",
        "run: echo ok\n      shell: bash\n      continue-on-error: '${{ secrets.TOLERATE }}'",
        "uses: actions/cache@v4\n      with: {key: '${{ secrets.KEY }}'}",
    ] {
        let action = format!(
            "name: Invalid context\ndescription: Invalid\nruns:\n  using: composite\n  steps:\n    - {field}\n"
        );
        assert!(!valid(&[("action", &action)], &[], "action"), "{field}");
    }

    let action = "name: Valid context\ndescription: Valid\nruns:\n  using: composite\n  steps:\n    - if: '${{ inputs.enabled }}'\n      run: 'echo ${{ github.ref }}'\n      shell: bash\n      env: {LABEL: '${{ vars.LABEL }}'}\n      continue-on-error: '${{ inputs.tolerate }}'\n    - uses: actions/cache@v4\n      with: {key: '${{ steps.setup.outputs.key }}'}\n";
    assert!(valid(&[("action", action)], &[], "action"));
}

#[test]
fn scalar_defaults_and_action_contexts_catalog_local_actions_for_the_step_scanner() {
    let action = "name: Cataloged\ndescription: Valid\ninputs:\n  retries: {description: Retries, default: 2}\nruns:\n  using: composite\n  steps:\n    - if: '${{ inputs.enabled }}'\n      run: 'echo ${{ vars.LABEL }}'\n      shell: bash\n";
    assert!(valid(
        &[(".github/actions/cataloged", action)],
        &[],
        ".github/actions/cataloged"
    ));

    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/typecheck.yml".to_string(),
            value: Ok(serde_yaml::from_str(
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./.github/actions/cataloged\n      - run: tsc --noEmit -p app/tsconfig.json\n",
            )
            .unwrap()),
        }],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);
    let tracked_paths = tracked
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    let project_source_inputs = ProjectSourceInputs::from([(
        "app/tsconfig.json".to_string(),
        BTreeSet::from(["app/tsconfig.json".to_string()]),
    )]);

    assert_eq!(
        super::super::ci_typechecked_projects_with_local_actions(
            std::path::Path::new("."),
            &workflows,
            &tracked,
            &tracked_paths,
            &project_source_inputs,
            &LocalActionCatalog::non_docker(BTreeSet::from([
                ".github/actions/cataloged".to_string(),
            ])),
        ),
        BTreeSet::from(["app/tsconfig.json".to_string()])
    );
}

#[test]
fn composite_actions_require_valid_acyclic_nested_local_targets() {
    let root = "name: Root\ndescription: Root\nruns: {using: composite, steps: [{uses: ./nested}]}";
    let nested = "name: Nested\ndescription: Nested\nruns: {using: composite, steps: [{run: ok, shell: bash}]}";
    assert!(valid(&[("root", root), ("nested", nested)], &[], "root"));
    assert!(!valid(&[("root", root)], &[], "root"));

    let cycle =
        "name: Nested\ndescription: Nested\nruns: {using: composite, steps: [{uses: ./root}]}";
    assert!(!valid(&[("root", root), ("nested", cycle)], &[], "root"));
}

#[test]
fn composite_actions_reject_unconditional_static_failures() {
    for (condition, run) in [
        ("", "false"),
        ("", "exit 1"),
        ("", "return"),
        ("", "false | true"),
        ("", "command false | true"),
        ("", "builtin false | true"),
        ("", "command builtin false | true"),
        ("", "builtin command false | true"),
        ("", "command -p -- false | true"),
        ("", "command -p -p false | true"),
        ("", "command -pp false | true"),
        ("if: true\n      ", "false"),
        ("if: '${{ vars.MAYBE }}'\n      ", "false"),
    ] {
        let action = format!(
            "name: Failing\ndescription: Failing\nruns:\n  using: composite\n  steps:\n    - {condition}shell: bash\n      run: '{run}'\n"
        );
        assert!(!valid(&[("action", &action)], &[], "action"), "{run}");
    }
    for step in [
        "{run: echo ok, shell: bash}",
        "{run: 'exit 0', shell: bash}",
        "{if: false, run: 'false', shell: bash}",
    ] {
        let action = format!(
            "name: Passing\ndescription: Passing\nruns: {{using: composite, steps: [{step}]}}"
        );
        assert!(valid(&[("action", &action)], &[], "action"), "{step}");
    }
    let static_shell_expression = "name: Failing\ndescription: Failing\nruns:\n  using: composite\n  steps:\n    - shell: \"${{ 'bash' }}\"\n      run: 'false | true'\n";
    assert!(!valid(
        &[("action", static_shell_expression)],
        &[],
        "action"
    ));
    let dynamic_shell = "name: Failing\ndescription: Failing\ninputs:\n  shell: {description: Shell, default: bash}\nruns:\n  using: composite\n  steps:\n    - shell: '${{ inputs.shell }}'\n      run: 'false | true'\n";
    assert!(!valid(&[("action", dynamic_shell)], &[], "action"));
}

#[test]
fn input_dependent_composite_conditions_cannot_hide_static_failures() {
    let action = "name: Conditional failure\ndescription: Conditional failure\ninputs:\n  enabled: {description: Enable failure}\nruns:\n  using: composite\n  steps:\n    - if: '${{ inputs.enabled }}'\n      run: 'false'\n      shell: bash\n";

    assert!(!valid(&[("action", action)], &[], "action"));
}

#[test]
fn action_pre_if_conditions_require_valid_documented_contexts() {
    let node = "name: Node\ndescription: Invalid\nruns: {using: node24, pre: setup.js, pre-if: \"runner.os == 'Linux' && always()\", main: dist/index.js}";
    assert!(!valid(
        &[("action", node)],
        &["action/setup.js", "action/dist/index.js"],
        "action"
    ));

    let docker = "name: Docker\ndescription: Valid\nruns: {using: docker, image: 'docker://alpine:3.22', pre-entrypoint: setup.sh, pre-if: \"github.ref != ''\"}";
    assert!(valid(&[("action", docker)], &[], "action"));

    for pre_if in [
        "runner.os ==",
        "steps.setup.outputs.ready",
        "hashFiles('**') != ''",
    ] {
        let docker = format!(
            "name: Docker\ndescription: Invalid\nruns: {{using: docker, image: 'docker://alpine:3.22', pre-entrypoint: setup.sh, pre-if: \"{pre_if}\"}}"
        );
        assert!(!valid(&[("action", &docker)], &[], "action"), "{pre_if}");
    }

    for post_if in [
        "runner.os ==",
        "steps.setup.outputs.ready",
        "hashFiles('**') != ''",
    ] {
        let node = format!(
            "name: Node\ndescription: Invalid\nruns: {{using: node24, main: dist/index.js, post: cleanup.js, post-if: \"{post_if}\"}}"
        );
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

#[test]
fn composite_action_nesting_is_bounded() {
    let descriptors = |count: usize| {
        (0..count)
            .map(|index| {
                let runs = if index + 1 == count {
                    "runs: {using: composite, steps: [{run: echo ok, shell: bash}]}".to_string()
                } else {
                    format!(
                        "runs: {{using: composite, steps: [{{uses: ./action-{}}}]}}",
                        index + 1
                    )
                };
                (
                    format!("action-{index}"),
                    serde_yaml::from_str(&format!(
                        "name: Action {index}\ndescription: Nested\n{runs}"
                    ))
                    .unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };

    assert!(action_directory_valid(
        "action-0",
        &descriptors(10),
        &BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    ));
    assert!(!action_directory_valid(
        "action-0",
        &descriptors(11),
        &BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    ));
}

#[test]
fn composite_run_working_directories_must_exist_in_the_checkout() {
    let action = |working_directory: &str, control: &str| {
        format!(
            "name: Composite\ndescription: Valid\nruns:\n  using: composite\n  steps:\n    - run: echo ok\n      shell: bash\n      working-directory: {working_directory}\n{control}"
        )
    };

    let existing = action("packages/app", "");
    assert!(valid(
        &[("action", &existing)],
        &["packages/app/src/index.ts"],
        "action"
    ));

    let literal_expression = action("\"packages/${{ 'app' }}\"", "");
    assert!(valid(
        &[("action", &literal_expression)],
        &["packages/app/src/index.ts"],
        "action"
    ));

    for missing in [
        action("packages/missing", ""),
        action("packages/app/src/index.ts", ""),
        action("../outside", ""),
    ] {
        assert!(!valid(
            &[("action", &missing)],
            &["packages/app/src/index.ts"],
            "action"
        ));
    }

    let checkout_root = action(".", "");
    assert!(valid(&[("action", &checkout_root)], &[], "action"));

    let dynamic = action("\"${{ inputs.directory }}\"", "");
    assert!(!valid(&[("action", &dynamic)], &[], "action"));

    let malformed = action("\"${{ github.ref == }}\"", "");
    assert!(!valid(&[("action", &malformed)], &[], "action"));

    for control in ["      if: false\n", "      continue-on-error: true\n"] {
        let ignored_missing = action("packages/missing", control);
        assert!(valid(&[("action", &ignored_missing)], &[], "action"));
    }
}
