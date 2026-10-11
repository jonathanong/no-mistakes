use super::super::artifact_types::ArtifactValue;
use super::super::artifact_values::artifact_value;
use super::super::value_primitives::OrderedJson;

#[test]
fn fixed_matrix_reference_pattern_is_shared_across_artifact_values() {
    let source = include_str!("../artifact_values/matrix_value.rs");
    assert!(source.contains("static MATRIX_REFERENCE_PATTERN: LazyLock<Regex>"));
    let artifact_value_body = source
        .split("pub fn artifact_value(")
        .nth(1)
        .and_then(|body| body.split("pub fn static_matrix_instance_count").next())
        .expect("artifact_value implementation must remain present");
    assert!(
        !artifact_value_body
            .contains("Regex::new(r\"\\$\\{\\{\\s*matrix\\.([A-Za-z_][\\w-]*)\\s*\\}\\}\")"),
        "artifact_value must not compile the fixed matrix reference regex per call"
    );
    assert!(artifact_value_body.contains("MATRIX_REFERENCE_PATTERN.captures_iter(raw)"));
    assert!(artifact_value_body.contains(".replace_all(raw, \"\")"));
    assert_eq!(
        artifact_value_body
            .matches("MATRIX_REFERENCE_PATTERN")
            .count(),
        2
    );
}

#[test]
fn shared_pattern_preserves_repeated_axis_expansion() {
    let axes = OrderedJson::Object(vec![(
        "os".to_string(),
        OrderedJson::Array(vec![
            OrderedJson::String("linux".to_string()),
            OrderedJson::String("macos".to_string()),
        ]),
    )]);
    let ArtifactValue::Finite { values, .. } =
        artifact_value("${{ matrix.os }}-${{ matrix.os }}", Some(&axes))
    else {
        panic!("expected a finite artifact value");
    };
    assert_eq!(values, vec!["linux-linux", "macos-macos"]);
}
