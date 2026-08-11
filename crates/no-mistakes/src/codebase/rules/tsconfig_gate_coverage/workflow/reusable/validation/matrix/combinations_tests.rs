use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::InputState;

fn job(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

fn static_matrix_combinations(job: &Value) -> Option<MatrixCombinations> {
    static_matrix_combinations_for_inputs(job, &InputState::new())
}

#[test]
fn combinations_distinguish_dynamic_expansion_and_malformed_includes() {
    for yaml in [
        "strategy:\n  matrix:\n    target: '${{ fromJSON(inputs.targets) }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    exclude: '${{ fromJSON(inputs.exclusions) }}'",
        "strategy:\n  matrix:\n    target: [linux]\n    include: '${{ fromJSON(inputs.inclusions) }}'",
    ] {
        assert!(matches!(
            static_matrix_combinations(&job(yaml)),
            Some(MatrixCombinations::Dynamic(_))
        ));
    }
    assert!(static_matrix_combinations(&job("strategy:\n  matrix:\n    target: []")).is_none());
    for yaml in [
        "strategy:\n  matrix:\n    target: ['${{ broken']",
        "strategy:\n  matrix:\n    target: [linux]\n    exclude: [invalid]",
        "strategy:\n  matrix:\n    target: [linux]\n    include: [invalid]",
    ] {
        assert!(static_matrix_combinations(&job(yaml)).is_none(), "{yaml}");
    }
    assert!(static_matrix_combinations(&job(
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - 1: invalid-key"
    ))
    .is_none());

    let axes = (0..20)
        .map(|index| format!("    axis{index}: [false, true]"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(static_matrix_combinations(&job(&format!("strategy:\n  matrix:\n{axes}"))).is_none());
}

#[test]
fn combinations_apply_includes_and_handle_empty_combinations() {
    let applied = static_matrix_combinations(&job(
        "strategy:\n  matrix:\n    target: [linux]\n    include:\n      - label: retained",
    ))
    .unwrap();
    assert_eq!(
        applied[0].get("target"),
        Some(&Value::String("linux".to_string()))
    );
    assert_eq!(
        applied[0].get("label"),
        Some(&Value::String("retained".to_string()))
    );

    let all_excluded = static_matrix_combinations(&job(
        "strategy:\n  matrix:\n    target: [linux]\n    exclude:\n      - target: linux",
    ))
    .unwrap();
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

#[test]
fn excessive_axis_depth_stops_before_recursive_traversal() {
    let axes = (0..=super::MAX_STATIC_MATRIX_AXIS_DEPTH)
        .map(|index| (format!("axis{index}"), vec![Value::Bool(false)]))
        .collect::<Vec<_>>();
    let mut matrix = serde_yaml::Mapping::new();
    for (name, choices) in &axes {
        matrix.insert(
            Value::String(name.clone()),
            Value::Sequence(choices.clone()),
        );
    }
    let job = Value::Mapping(serde_yaml::Mapping::from_iter([(
        Value::String("strategy".into()),
        Value::Mapping(serde_yaml::Mapping::from_iter([(
            Value::String("matrix".into()),
            Value::Mapping(matrix),
        )])),
    )]));
    assert!(!matrix_shape_valid(&job));
    assert!(static_matrix_combinations(&job).is_none());

    let mut values = BTreeMap::new();
    let mut states_remaining = 1_000_000;
    assert_eq!(
        traversal::has_applicable_combination(
            &axes,
            &[],
            &serde_yaml::Mapping::new(),
            0,
            &mut values,
            &mut states_remaining,
        ),
        None
    );
}

#[test]
fn static_mappings_classify_empty_literal_dynamic_and_invalid_values() {
    use super::mappings::{static_mappings, StaticMappings};

    let literal = job("include:\n  - target: \"${{ 'linux' }}\"\n    attempts: '${{ 2 }}'");
    assert!(matches!(
        static_mappings(literal.get("include")),
        StaticMappings::Static(values) if values.len() == 1
    ));
    for yaml in [
        "include: []",
        "include: invalid",
        "include:\n  - invalid",
        "include:\n  - target: '${{ broken'",
        "include: '${{ true }}'",
    ] {
        assert!(
            matches!(
                static_mappings(job(yaml).get("include")),
                StaticMappings::Invalid
            ),
            "{yaml}"
        );
    }
    for yaml in [
        "include: '${{ fromJSON(inputs.include) }}'",
        "include:\n  - target: '${{ inputs.target }}'",
    ] {
        assert!(
            matches!(
                static_mappings(job(yaml).get("include")),
                StaticMappings::Dynamic
            ),
            "{yaml}"
        );
    }
    assert!(matches!(static_mappings(None), StaticMappings::Static(values) if values.is_empty()));
}
