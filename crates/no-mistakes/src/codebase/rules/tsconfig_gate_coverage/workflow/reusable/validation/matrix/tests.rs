use super::*;

fn job(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn uniform_matrix_values_follow_exclude_and_include_expansion() {
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
        assert_eq!(
            uniform_static_matrix_values(&job(yaml)).get("enabled"),
            expected.as_ref(),
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
            !uniform_static_matrix_values(&job(yaml)).contains_key("enabled"),
            "{yaml}"
        );
    }
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
    ] {
        assert!(!matrix_shape_valid(&job(yaml)), "{yaml}");
    }
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
fn uniform_values_fail_closed_for_unresolvable_expansion_and_malformed_includes() {
    for yaml in [
        "strategy:\n  matrix:\n    target: []",
        "strategy:\n  matrix:\n    target: '${{ fromJSON(inputs.targets) }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    exclude: '${{ fromJSON(inputs.exclusions) }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    include: '${{ fromJSON(inputs.inclusions) }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - 1: invalid-key",
    ] {
        assert!(uniform_static_matrix_values(&job(yaml)).is_empty(), "{yaml}");
    }

    let axes = (0..20)
        .map(|index| format!("    axis{index}: [false, true]"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        uniform_static_matrix_values(&job(&format!("strategy:\n  matrix:\n{axes}"))).is_empty()
    );
}

#[test]
fn uniform_values_apply_includes_and_handle_empty_combinations() {
    let applied = uniform_static_matrix_values(&job(
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - label: retained",
    ));
    assert_eq!(
        applied.get("target"),
        Some(&Value::String("linux".to_string()))
    );
    assert_eq!(
        applied.get("label"),
        Some(&Value::String("retained".to_string()))
    );

    let all_excluded = uniform_static_matrix_values(&job(
        "strategy:\n  matrix:\n    target: [linux]\n    exclude:\n      - target: linux",
    ));
    assert!(all_excluded.is_empty());
}

#[test]
fn traversal_exhaustion_and_nonmatching_includes_are_conservative() {
    let axes = vec![(
        "target".to_string(),
        vec![Value::String("linux".to_string())],
    )];
    let include = serde_yaml::from_str::<Value>("target: macos")
        .unwrap()
        .as_mapping()
        .unwrap()
        .clone();
    let mut values = BTreeMap::new();
    let mut exhausted = 0;
    assert_eq!(
        traversal::has_applicable_combination(&axes, &[], &include, 0, &mut values, &mut exhausted,),
        None
    );

    let mut states = 8;
    assert_eq!(
        traversal::has_applicable_combination(&axes, &[], &include, 0, &mut values, &mut states,),
        Some(false)
    );
}
