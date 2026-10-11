use super::super::artifact_types::ArtifactValue;
use super::super::artifact_values::{artifact_value, static_matrix_instance_count};
use super::super::value_primitives::OrderedJson;

#[test]
fn empty_matrix_has_one_combination_without_an_instance_multiplier() {
    let matrix = OrderedJson::Object(Vec::new());
    assert_eq!(static_matrix_instance_count(Some(&matrix)), Some(1));
    assert_eq!(
        artifact_value("literal", Some(&matrix)),
        ArtifactValue::Static {
            raw: "literal".to_string(),
            value: "literal".to_string(),
            instance_count: None,
        }
    );
}

#[test]
fn static_matrix_count_and_value_use_last_duplicate_axis_after_raw_cap_check() {
    let axes = OrderedJson::Object(vec![
        (
            "os".to_string(),
            OrderedJson::Array(vec![
                OrderedJson::String("old-linux".to_string()),
                OrderedJson::String("old-macos".to_string()),
            ]),
        ),
        (
            "os".to_string(),
            OrderedJson::Array(vec![
                OrderedJson::String("linux".to_string()),
                OrderedJson::String("macos".to_string()),
                OrderedJson::String("windows".to_string()),
            ]),
        ),
    ]);

    assert_eq!(static_matrix_instance_count(Some(&axes)), Some(3));
    assert_eq!(
        artifact_value("literal", Some(&axes)),
        ArtifactValue::Static {
            raw: "literal".to_string(),
            value: "literal".to_string(),
            instance_count: Some(3),
        }
    );
    assert_eq!(
        artifact_value("${{ matrix.os }}", Some(&axes)),
        ArtifactValue::Finite {
            raw: "${{ matrix.os }}".to_string(),
            values: vec![
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string()
            ],
            instance_counts: [
                ("linux".to_string(), 1),
                ("macos".to_string(), 1),
                ("windows".to_string(), 1),
            ]
            .into(),
        }
    );
}

#[test]
fn duplicate_axis_raw_product_still_obeys_matrix_cap() {
    let axes = OrderedJson::Object(vec![
        (
            "os".to_string(),
            OrderedJson::Array(
                (0..16)
                    .map(|value| OrderedJson::Number(value.into()))
                    .collect(),
            ),
        ),
        (
            "os".to_string(),
            OrderedJson::Array(
                (0..17)
                    .map(|value| OrderedJson::Number(value.into()))
                    .collect(),
            ),
        ),
    ]);

    assert_eq!(static_matrix_instance_count(Some(&axes)), None);
    assert_eq!(
        artifact_value("literal", Some(&axes)),
        ArtifactValue::Static {
            raw: "literal".to_string(),
            value: "literal".to_string(),
            instance_count: None,
        }
    );
    assert_eq!(
        artifact_value("${{ matrix.os }}", Some(&axes)),
        ArtifactValue::Dynamic {
            raw: "${{ matrix.os }}".to_string(),
        }
    );
}

#[test]
fn static_matrix_count_borrows_axes_and_does_not_materialize_values() {
    let source = include_str!("../artifact_values/matrix_value.rs");
    let artifact_value_body = source
        .split("pub fn artifact_value(")
        .nth(1)
        .and_then(|body| body.split("pub fn static_matrix_instance_count").next())
        .expect("artifact_value implementation must remain present");
    let static_branch_end = artifact_value_body
        .find("if !raw.contains(\"${{\")")
        .expect("static artifact branch must remain present");
    let static_return = artifact_value_body
        .find("return ArtifactValue::Static")
        .expect("static artifact branch must return a static value");
    let axis_materialization = artifact_value_body
        .find("matrix_axis_values(axes)")
        .expect("dynamic expansion must materialize replacement values");
    assert!(static_branch_end < axis_materialization);
    assert!(static_return < axis_materialization);
    assert!(
        !artifact_value_body[..static_branch_end].contains("matrix_axis_values"),
        "static artifact values must not allocate owned axis strings"
    );

    let count_body = source
        .split("pub fn static_matrix_instance_count(")
        .nth(1)
        .expect("matrix count implementation must remain present");
    assert!(count_body.contains("simple_matrix_axes(matrix)"));
    assert!(count_body.contains("matrix_combination_count(&axes)"));
    assert!(
        !count_body.contains("matrix_axis_values"),
        "matrix count must not materialize owned axis strings"
    );
    let axes_source = include_str!("../artifact_values/matrix_value/axes.rs");
    assert!(axes_source.contains("BTreeMap<&'a str, &'a [OrderedJson]>"));
}
