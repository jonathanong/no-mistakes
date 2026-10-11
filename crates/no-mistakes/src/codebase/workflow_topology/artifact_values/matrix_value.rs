//! Matrix-aware artifact value resolution, split out of [`super`] to stay
//! under the crate's per-file line limit. Re-exported by [`super`] so
//! `artifact_values::artifact_value` / `artifact_values::static_matrix_instance_count`
//! keep working unchanged for every external caller.

use super::super::artifact_types::ArtifactValue;
use super::super::value_primitives::OrderedJson;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

mod axes;
use axes::{matrix_axis_values, matrix_combination_count, simple_matrix_axes};

static MATRIX_REFERENCE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{\{\s*matrix\.([A-Za-z_][\w-]*)\s*\}\}")
        .expect("well-formed matrix reference regex")
});

/// Resolves a raw string (an artifact `name`/`pattern`/`artifact-ids`/
/// `repository`/`run-id`) against an optional job matrix: a literal with no
/// `${{ }}` is `static`; one referencing only simple matrix axes expands to
/// every combination (`finite`, deduplicated, with an instance count per
/// value accounting for axes the string doesn't reference); anything else
/// is `dynamic` (unresolvable without running the job).
pub fn artifact_value(raw: &str, matrix: Option<&OrderedJson>) -> ArtifactValue {
    let axes = simple_matrix_axes(matrix);
    if !raw.contains("${{") {
        let instance_count = axes.as_ref().map(matrix_combination_count).unwrap_or(1);
        return ArtifactValue::Static {
            raw: raw.to_string(),
            value: raw.to_string(),
            instance_count: (instance_count > 1).then_some(instance_count),
        };
    }

    let mut referenced_axes: Vec<String> = Vec::new();
    for captures in MATRIX_REFERENCE_PATTERN.captures_iter(raw) {
        let axis = captures[1].to_string();
        if !referenced_axes.contains(&axis) {
            referenced_axes.push(axis);
        }
    }
    let Some(axes) = axes.filter(|_| !referenced_axes.is_empty()) else {
        return ArtifactValue::Dynamic {
            raw: raw.to_string(),
        };
    };
    if referenced_axes
        .iter()
        .any(|axis| !axes.contains_key(axis.as_str()))
    {
        return ArtifactValue::Dynamic {
            raw: raw.to_string(),
        };
    }
    if MATRIX_REFERENCE_PATTERN
        .replace_all(raw, "")
        .contains("${{")
    {
        return ArtifactValue::Dynamic {
            raw: raw.to_string(),
        };
    }

    let axes = matrix_axis_values(axes).expect("validated matrix axes contain only scalar values");
    let mut expanded_values = vec![raw.to_string()];
    for axis in &referenced_axes {
        let items = &axes[axis.as_str()];
        if items.iter().any(|item| item.contains('$')) {
            // `replace_all` interprets `$0`/`$1`/`$$` in its replacement
            // string. Retain that behavior for axes whose values need it.
            let expression = Regex::new(&format!(
                r"\$\{{\{{\s*matrix\.{}\s*\}}\}}",
                regex::escape(axis)
            ))
            .expect("well-formed regex");
            expanded_values = expanded_values
                .into_iter()
                .flat_map(|value| {
                    items
                        .iter()
                        .map(|item| expression.replace_all(&value, item.as_str()).into_owned())
                        .collect::<Vec<_>>()
                })
                .collect();
        } else {
            expanded_values = expanded_values
                .into_iter()
                .flat_map(|value| {
                    items
                        .iter()
                        .map(|item| {
                            MATRIX_REFERENCE_PATTERN
                                .replace_all(&value, |captures: &regex::Captures<'_>| {
                                    if &captures[1] == axis.as_str() {
                                        item.clone()
                                    } else {
                                        captures[0].to_string()
                                    }
                                })
                                .into_owned()
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
        }
    }

    let omitted_axis_multiplier: u32 = axes
        .iter()
        .filter(|(axis, _)| {
            !referenced_axes
                .iter()
                .any(|referenced| referenced.as_str() == **axis)
        })
        .map(|(_, values)| values.len() as u32)
        .product();
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for value in expanded_values {
        *counts.entry(value).or_insert(0) += omitted_axis_multiplier;
    }
    let values: Vec<String> = counts.keys().cloned().collect();
    ArtifactValue::Finite {
        raw: raw.to_string(),
        values,
        instance_counts: counts,
    }
}

pub fn static_matrix_instance_count(matrix: Option<&OrderedJson>) -> Option<u32> {
    match simple_matrix_axes(matrix) {
        Some(axes) => Some(matrix_combination_count(&axes)),
        None if matrix.is_none() => Some(1),
        None => None,
    }
}
