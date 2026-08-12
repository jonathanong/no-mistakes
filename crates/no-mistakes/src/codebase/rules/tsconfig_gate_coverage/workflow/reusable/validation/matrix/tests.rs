use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::InputState;

fn static_matrix_combinations(job: &Value) -> Option<MatrixCombinations> {
    static_matrix_combinations_for_inputs(job, &InputState::new())
}

fn zero_instance_matrix(job: &Value) -> bool {
    matches!(
        static_matrix_combinations(job),
        Some(MatrixCombinations::Static(values)) if values.is_empty()
    )
}

fn job(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn static_matrix_combinations_follow_exclude_and_include_expansion() {
    for (yaml, expected) in [
        (
            "strategy:\n  matrix:\n    enabled: [false]",
            Some(Value::Bool(false)),
        ),
        (
            "strategy:\n  matrix:\n    enabled: [true, true]\n    os: [linux, macos]",
            Some(Value::Bool(true)),
        ),
        (
            "strategy:\n  matrix:\n    enabled: [false, true]\n    exclude:\n      - enabled: true",
            Some(Value::Bool(false)),
        ),
        (
            "strategy:\n  matrix:\n    include:\n      - enabled: false\n      - enabled: false",
            Some(Value::Bool(false)),
        ),
    ] {
        let combinations = static_matrix_combinations(&job(yaml)).unwrap();
        assert!(
            combinations
                .iter()
                .all(|combination| combination.get("enabled") == expected.as_ref()),
            "{yaml}"
        );
    }
    for yaml in [
        "strategy:\n  matrix:\n    enabled: [false, true]",
        "strategy:\n  matrix:\n    enabled: [false]\n    include:\n      - enabled: true",
        "strategy:\n  matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'",
        "strategy:\n  matrix:\n    enabled: ['${{ inputs.enabled }}']",
    ] {
        assert!(
            static_matrix_combinations(&job(yaml))
                .unwrap()
                .iter()
                .any(|combination| !combination.contains_key("enabled"))
                || static_matrix_combinations(&job(yaml)).unwrap().len() > 1,
            "{yaml}"
        );
    }
}

#[test]
fn literal_expression_mapping_values_match_typed_matrix_values() {
    let matrix = job(
        "strategy:\n  matrix:\n    boolean: [true]\n    number: [7]\n    string: [release]\n    nullable: [null]\n    exclude:\n      - boolean: '${{ true }}'\n        number: '${{ 7 }}'\n        string: \"${{ 'release' }}\"\n        nullable: '${{ null }}'",
    );
    assert!(zero_instance_matrix(&matrix));
    assert!(matches!(
        static_matrix_combinations(&matrix),
        Some(MatrixCombinations::Static(values)) if values.is_empty()
    ));

    let included = job(
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - target: \"${{ 'linux' }}\"\n        enabled: '${{ false }}'\n        attempts: '${{ 2 }}'\n        optional: '${{ null }}'",
    );
    assert_eq!(
        static_matrix_combinations(&included),
        Some(MatrixCombinations::Static(vec![BTreeMap::from([
            ("attempts".to_string(), Value::Number(2.into())),
            ("enabled".to_string(), Value::Bool(false)),
            ("optional".to_string(), Value::Null),
            ("target".to_string(), Value::String("linux".to_string())),
        ])]))
    );
}

#[test]
fn literal_expression_axis_values_expand_as_their_resolved_yaml_values() {
    let matrix = job(
        "strategy:\n  matrix:\n    enabled: ['${{ true }}']\n    attempts: ['${{ 2 }}']\n    target: [\"${{ 'linux' }}\"]\n    optional: ['${{ null }}']",
    );

    assert!(matrix_shape_valid(&matrix));
    assert_eq!(
        static_matrix_combinations(&matrix),
        Some(MatrixCombinations::Static(vec![BTreeMap::from([
            ("attempts".to_string(), Value::Number(2.into())),
            ("enabled".to_string(), Value::Bool(true)),
            ("optional".to_string(), Value::Null),
            ("target".to_string(), Value::String("linux".to_string())),
        ])]))
    );
}

#[test]
fn dynamic_mapping_expressions_do_not_enumerate_static_combinations() {
    let matrix = job(
        "strategy:\n  matrix:\n    target: [linux]\n    exclude:\n      - target: '${{ inputs.target }}'",
    );
    assert!(!zero_instance_matrix(&matrix));
    assert!(matches!(
        static_matrix_combinations(&matrix),
        Some(MatrixCombinations::Dynamic(_))
    ));
}

#[test]
fn dynamic_matrices_fail_open_and_malformed_shapes_fail_closed() {
    assert!(!zero_instance_matrix(&job("strategy:\n  matrix: {}")));
    assert!(!zero_instance_matrix(&job(
        "strategy:\n  matrix:\n    target:\n      - [nested]"
    )));
    assert!(!zero_instance_matrix(&job(
        "strategy:\n  matrix:\n    target: [linux]\n    exclude: invalid"
    )));
    assert!(!zero_instance_matrix(&job(
        "strategy:\n  matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'"
    )));
    assert!(!matrix_shape_valid(&job("strategy: []")));
    assert!(!matrix_shape_valid(&job("strategy:\n  matrix: false")));
    assert!(matrix_shape_valid(&job("strategy:\n  fail-fast: false")));
    assert!(!matrix_shape_valid(&job("strategy:\n  matrix: static")));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix: ' ${{ fromJSON(needs.setup.outputs.matrix) }} '"
    )));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix:\n    os: '${{ fromJSON(needs.setup.outputs.operating_systems) }}'"
    )));
    for yaml in [
        "strategy:\n  matrix:\n    os: ubuntu-latest",
        "strategy:\n  matrix:\n    os: true",
        "strategy:\n  matrix:\n    os: {name: ubuntu-latest}",
        "strategy:\n  matrix:\n    1: [ubuntu-latest]",
        "strategy:\n  matrix:\n    os: [ubuntu-latest]\n    include: true",
        "strategy:\n  matrix:\n    os: [ubuntu-latest]\n    exclude: [invalid]",
        "strategy:\n  matrix: {}",
        "strategy:\n  matrix:\n    target: []",
        "strategy:\n  matrix:\n    target: [ubuntu-latest]\n    include: []",
        "strategy:\n  matrix:\n    target: [ubuntu-latest]\n    include: [invalid]",
        "strategy:\n  matrix:\n    target: [ubuntu-latest]\n    exclude: []",
        "strategy:\n  matrix:\n    target: '${{ fromJSON(needs.setup.outputs.targets) }}'\n    include: []",
        "strategy:\n  matrix:\n    target: '${{ fromJSON(needs.setup.outputs.targets) }}'\n    include: true",
        "strategy:\n  matrix:\n    target: '${{ fromJSON(needs.setup.outputs.targets) }}'\n    exclude: []",
        "strategy:\n  matrix:\n    target: '${{ fromJSON(needs.setup.outputs.targets) }}'\n    exclude: invalid",
        "strategy:\n  matrix:\n    target: [ubuntu-latest]\n    include: '${{ true }}'",
        "strategy:\n  matrix:\n    target: [ubuntu-latest]\n    exclude: '${{ null }}'",
        "strategy:\n  matrix:\n    target: [ubuntu-latest]\n    include: \"${{ contains('matrix', 'm') }}\"",
    ] {
        assert!(!matrix_shape_valid(&job(yaml)), "{yaml}");
    }
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix:\n    os: [ubuntu-latest]\n    include: '${{ fromJSON(needs.setup.outputs.include) }}'"
    )));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix:\n    include:\n      - target: ubuntu-latest"
    )));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix:\n    include: '${{ fromJSON(needs.setup.outputs.include) }}'"
    )));
    let all_excluded = job(
        "strategy:\n  matrix:\n    target: [ubuntu-latest]\n    exclude:\n      - target: ubuntu-latest",
    );
    assert!(matrix_shape_valid(&all_excluded));
    assert!(zero_instance_matrix(&all_excluded));
}

#[test]
fn dynamic_matrix_requires_one_nonempty_expression() {
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'"
    )));
    for yaml in [
        "strategy:\n  matrix: '${{ }}'",
        "strategy:\n  matrix: '${{ true }}${{ false }}'",
        "strategy:\n  matrix: '${{ true }}}'",
        "strategy:\n  matrix: '${{{ true }}'",
        "strategy:\n  matrix: '${{ jobs.build.result }}'",
    ] {
        assert!(!matrix_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn matrix_expressions_require_strategy_matrix_contexts() {
    for yaml in [
        "strategy:\n  matrix:\n    target: ['${{ jobs.build.result }}']",
        "strategy:\n  matrix:\n    target: ['linux-${{ jobs.build.result }}-suffix']",
        "strategy:\n  matrix:\n    target: ['${{ inputs.target }}', '${{ jobs.build.result }}']",
        "strategy:\n  matrix:\n    target: ['${{ jobs.build.result }}', '${{ inputs.target }}']",
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - target: '${{ steps.setup.outputs.target }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - target: 'linux-${{ steps.setup.outputs.target }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - target: '${{ inputs.target }}'\n        label: '${{ jobs.build.result }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - target: '${{ inputs.target }}'\n      - target: '${{ jobs.build.result }}'",
        "strategy:\n  matrix:\n    target: '${{ inputs.targets }}'\n    include:\n      - target: '${{ jobs.build.result }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    exclude: '${{ secrets.EXCLUSIONS }}'",
    ] {
        let job = job(yaml);
        assert!(!matrix_shape_valid(&job), "{yaml}");
        assert!(static_matrix_combinations(&job).is_none(), "{yaml}");
    }

    for yaml in [
        "strategy:\n  matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'",
        "strategy:\n  matrix:\n    target: ['${{ inputs.target }}']",
        "strategy:\n  matrix:\n    target: ['linux-${{ inputs.target }}']",
    ] {
        assert!(matrix_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn root_matrix_expression_requires_a_dynamic_result() {
    for expression in [
        "true",
        "42",
        "'matrix'",
        "null",
        "contains('matrix', 'm')",
        "startsWith('matrix', 'm')",
        "success()",
        "toJSON(github)",
        "true || false",
        "case(inputs.enabled, fromJSON(inputs.matrix), needs.setup.outputs.matrix)",
    ] {
        let yaml = format!("strategy:\n  matrix: \"${{{{ {expression} }}}}\"");
        let job = job(&yaml);
        assert!(!matrix_shape_valid(&job), "{expression}");
        assert!(static_matrix_combinations(&job).is_none(), "{expression}");
    }

    for expression in [
        "fromJSON(needs.setup.outputs.matrix)",
        "needs.setup.outputs.matrix",
    ] {
        let yaml = format!("strategy:\n  matrix: \"${{{{ {expression} }}}}\"");
        let dynamic = job(&yaml);
        assert!(matrix_shape_valid(&dynamic), "{expression}");
        assert!(
            matches!(
                static_matrix_combinations(&dynamic),
                Some(MatrixCombinations::Dynamic(_))
            ),
            "{expression}"
        );
    }
}

#[test]
fn root_matrix_rejects_literal_non_mapping_from_json_values() {
    for expression in [
        "fromJSON('null')",
        "fromJSON('true')",
        "fromJSON('42')",
        "fromJSON('\\\"matrix\\\"')",
        "fromJSON('[\\\"linux\\\"]')",
    ] {
        let yaml = format!("strategy:\n  matrix: \"${{{{ {expression} }}}}\"");
        assert!(!matrix_shape_valid(&job(&yaml)), "{expression}");
    }
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix: \"${{ fromJSON('{\\\"os\\\":[\\\"ubuntu-latest\\\"]}') }}\""
    )));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix: \"${{ fromJSON(needs.setup.outputs.matrix) }}\""
    )));
}

#[test]
fn static_matrix_shape_enforces_the_github_job_limit() {
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix:\n    a: [1, 2]\n    b: [3, 4]"
    )));
    let values = (0..257)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    assert!(!matrix_shape_valid(&job(&format!(
        "strategy:\n  matrix:\n    value: [{values}]"
    ))));
    assert!(!matrix_shape_valid(&job(&format!(
        "strategy:\n  matrix:\n    value: [{values}]\n    include:\n      - label: ignored-after-limit"
    ))));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'"
    )));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix:\n    target:\n      - [nested]"
    )));

    let values = (0..257)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    assert!(matrix_shape_valid(&job(&format!(
        "strategy:\n  matrix:\n    value: [{values}]\n    exclude:\n      - value: 256"
    ))));

    let includes = (0..257)
        .map(|value| format!("      - value: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!matrix_shape_valid(&job(&format!(
        "strategy:\n  matrix:\n    include:\n{includes}"
    ))));

    let values = (0..256)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    assert!(!matrix_shape_valid(&job(&format!(
        "strategy:\n  matrix:\n    value: [{values}]\n    include:\n      - value: 256"
    ))));

    let objects = (0..257)
        .map(|value| format!("{{id: {value}}}"))
        .collect::<Vec<_>>()
        .join(", ");
    assert!(!matrix_shape_valid(&job(&format!(
        "strategy:\n  matrix:\n    target: [{objects}]"
    ))));
    assert!(matrix_shape_valid(&job(
        "strategy:\n  matrix:\n    value: [1, 2]\n    exclude:\n      - value: 1\n    include:\n      - value: 1\n      - value: 2\n        label: retained"
    )));
}

#[test]
fn include_matching_does_not_leak_prior_axis_assignments() {
    let matrix = job(
        "strategy:\n  matrix:\n    a: [1, 2]\n    b: [1, 2]\n    exclude:\n      - {a: 1, b: 1}\n      - {a: 1, b: 2}\n      - {a: 2, b: 2}\n    include:\n      - {label: retained}",
    );
    assert!(matches!(
        static_matrix_job_count(
            matrix
                .get("strategy")
                .and_then(|strategy| strategy.get("matrix"))
                .and_then(Value::as_mapping)
                .unwrap()
        ),
        StaticMatrixJobCount::Known(1)
    ));
}

#[test]
fn bounded_static_matrix_enumeration_rejects_unresolved_literal_expansions() {
    let axes = (0..20)
        .map(|index| format!("    axis{index}: [false, true]"))
        .collect::<Vec<_>>()
        .join("\n");
    let matrix = format!(
        "strategy:\n  matrix:\n{axes}\n    exclude:\n      - axis19: false\n      - axis19: true"
    );

    assert!(!matrix_shape_valid(&job(&matrix)));
}

#[test]
fn static_matrix_axis_depth_is_bounded_before_recursive_traversal() {
    let matrix_with_axes = |axis_count| {
        let axes = (0..axis_count)
            .map(|index| format!("    axis{index}: [only]"))
            .collect::<Vec<_>>()
            .join("\n");
        job(&format!("strategy:\n  matrix:\n{axes}"))
    };

    let at_limit = matrix_with_axes(MATRIX_JOB_LIMIT);
    assert!(matrix_shape_valid(&at_limit));
    assert!(static_matrix_combinations(&at_limit).is_some());

    let matrix = matrix_with_axes(10_000);

    assert!(!matrix_shape_valid(&matrix));
    assert!(static_matrix_combinations(&matrix).is_none());
}
