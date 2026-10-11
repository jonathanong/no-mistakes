use super::super::artifact_types::ArtifactValue;
use super::super::artifact_values::artifact_value;
use super::super::value_primitives::OrderedJson;
use std::collections::BTreeMap;

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
    let (before_dollar_check, dollar_fallback) = artifact_value_body
        .split_once("if items.iter().any(|item| item.contains('$')) {")
        .expect("per-axis dollar fallback guard must remain present");
    let (dollar_fallback, plain_fast_path) = dollar_fallback
        .split_once("} else {")
        .expect("plain-value fast path must remain the fallback branch");
    assert!(!before_dollar_check.contains("Regex::new("));
    assert_eq!(dollar_fallback.matches("Regex::new(").count(), 1);
    assert!(!plain_fast_path.contains("Regex::new("));
    assert!(plain_fast_path.contains("MATRIX_REFERENCE_PATTERN"));
    assert!(plain_fast_path.contains(".replace_all(&value, |captures:"));
    assert_eq!(
        artifact_value_body
            .matches("MATRIX_REFERENCE_PATTERN")
            .count(),
        3
    );
}

#[test]
fn plain_axis_fast_path_handles_whitespace_and_hyphenated_names() {
    let axes = OrderedJson::Object(vec![(
        "build-target".to_string(),
        OrderedJson::Array(vec![
            OrderedJson::String("linux".to_string()),
            OrderedJson::String("macos".to_string()),
        ]),
    )]);
    let raw = "${{matrix.build-target}}-${{  matrix.build-target   }}";
    let expected = legacy_matrix_expansion(raw, &axes);
    let ArtifactValue::Finite { values, .. } = artifact_value(raw, Some(&axes)) else {
        panic!("expected a finite artifact value");
    };
    assert_eq!(values, vec!["linux-linux", "macos-macos"]);
    assert_eq!(values, expected.keys().cloned().collect::<Vec<_>>());
}

#[test]
fn dollar_axis_fallback_preserves_regex_replacement_syntax() {
    let axes = OrderedJson::Object(vec![
        (
            "os".to_string(),
            OrderedJson::Array(vec![
                OrderedJson::String("$0".to_string()),
                OrderedJson::String("$1".to_string()),
                OrderedJson::String("$$".to_string()),
            ]),
        ),
        (
            "node".to_string(),
            OrderedJson::Array(vec![OrderedJson::String("20".to_string())]),
        ),
    ]);
    let raw = "release-${{ matrix.os }}-${{ matrix.os }}-${{ matrix.node }}";
    let expected = legacy_matrix_expansion(raw, &axes);
    let ArtifactValue::Finite {
        values,
        instance_counts,
        ..
    } = artifact_value(raw, Some(&axes))
    else {
        panic!("expected a finite artifact value");
    };
    assert_eq!(values, expected.keys().cloned().collect::<Vec<_>>());
    assert_eq!(instance_counts, expected);
}

#[test]
fn an_axis_value_can_introduce_a_reference_for_a_later_axis() {
    let axes = OrderedJson::Object(vec![
        (
            "os".to_string(),
            OrderedJson::Array(vec![
                // Regex replacement's `$$` escape emits a literal `$`; the
                // later node pass then sees the resulting matrix reference.
                OrderedJson::String("$${{ matrix.node }}".to_string()),
                OrderedJson::String("runner".to_string()),
            ]),
        ),
        (
            "node".to_string(),
            OrderedJson::Array(vec![
                OrderedJson::String("20".to_string()),
                OrderedJson::String("22".to_string()),
            ]),
        ),
    ]);
    let raw = "${{ matrix.os }}-${{ matrix.node }}";
    let expected = legacy_matrix_expansion(raw, &axes);
    let ArtifactValue::Finite {
        values,
        instance_counts,
        ..
    } = artifact_value(raw, Some(&axes))
    else {
        panic!("expected a finite artifact value");
    };
    assert_eq!(values, vec!["20-20", "22-22", "runner-20", "runner-22"]);
    assert_eq!(values, expected.keys().cloned().collect::<Vec<_>>());
    assert_eq!(instance_counts, expected);
}

fn legacy_matrix_expansion(raw: &str, axes: &OrderedJson) -> BTreeMap<String, u32> {
    let OrderedJson::Object(axes) = axes else {
        panic!("test axes must be an object");
    };
    let mut referenced_axes = Vec::new();
    for captures in regex::Regex::new(r"\$\{\{\s*matrix\.([A-Za-z_][\w-]*)\s*\}\}")
        .unwrap()
        .captures_iter(raw)
    {
        if !referenced_axes.contains(&captures[1].to_string()) {
            referenced_axes.push(captures[1].to_string());
        }
    }
    let mut expanded = vec![raw.to_string()];
    for axis in referenced_axes {
        let regex = regex::Regex::new(&format!(
            r"\$\{{\{{\s*matrix\.{}\s*\}}\}}",
            regex::escape(&axis)
        ))
        .unwrap();
        let Some((_, OrderedJson::Array(items))) = axes.iter().find(|(key, _)| key == &axis) else {
            panic!("test axis values must be arrays");
        };
        expanded = expanded
            .into_iter()
            .flat_map(|value| {
                items
                    .iter()
                    .map(|item| {
                        let OrderedJson::String(item) = item else {
                            panic!("test axis values must be strings");
                        };
                        regex.replace_all(&value, item.as_str()).into_owned()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
    }
    let mut counts = BTreeMap::new();
    for value in expanded {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
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
