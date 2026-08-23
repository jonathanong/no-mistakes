//! Matrix-aware artifact value resolution, split out of [`super`] to stay
//! under the crate's per-file line limit. Re-exported by [`super`] so
//! `artifact_values::artifact_value` / `artifact_values::static_matrix_instance_count`
//! keep working unchanged for every external caller.

use super::super::artifact_types::ArtifactValue;
use super::super::value_primitives::OrderedJson;
use regex::Regex;
use std::collections::BTreeMap;

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

    let reference_pattern =
        Regex::new(r"\$\{\{\s*matrix\.([A-Za-z_][\w-]*)\s*\}\}").expect("well-formed regex");
    let mut referenced_axes: Vec<String> = Vec::new();
    for captures in reference_pattern.captures_iter(raw) {
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
    if referenced_axes.iter().any(|axis| !axes.contains_key(axis)) {
        return ArtifactValue::Dynamic {
            raw: raw.to_string(),
        };
    }
    if reference_pattern.replace_all(raw, "").contains("${{") {
        return ArtifactValue::Dynamic {
            raw: raw.to_string(),
        };
    }

    let mut expanded_values = vec![raw.to_string()];
    for axis in &referenced_axes {
        let expression = Regex::new(&format!(
            r"\$\{{\{{\s*matrix\.{}\s*\}}\}}",
            regex::escape(axis)
        ))
        .expect("well-formed regex");
        let items = &axes[axis];
        expanded_values = expanded_values
            .into_iter()
            .flat_map(|value| {
                items
                    .iter()
                    .map(|item| expression.replace_all(&value, item.as_str()).into_owned())
                    .collect::<Vec<_>>()
            })
            .collect();
    }

    let omitted_axis_multiplier: u32 = axes
        .iter()
        .filter(|(axis, _)| !referenced_axes.contains(axis))
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

fn matrix_combination_count(axes: &BTreeMap<String, Vec<String>>) -> u32 {
    axes.values().map(|values| values.len() as u32).product()
}

/// A matrix's axes as simple string-value lists, when every axis is a
/// non-empty array of scalars and the matrix has no `include`/`exclude`
/// (which make the real combination set impossible to enumerate this way).
/// `None` for anything else, including a combination count over 256 (the
/// documented GitHub Actions job-matrix cap).
fn simple_matrix_axes(matrix: Option<&OrderedJson>) -> Option<BTreeMap<String, Vec<String>>> {
    let OrderedJson::Object(entries) = matrix? else {
        return None;
    };
    if entries
        .iter()
        .any(|(key, _)| key == "include" || key == "exclude")
    {
        return None;
    }
    let mut axes = BTreeMap::new();
    let mut combinations: u64 = 1;
    for (axis, value) in entries {
        let OrderedJson::Array(items) = value else {
            return None;
        };
        if items.is_empty() {
            return None;
        }
        let mut values = Vec::with_capacity(items.len());
        for item in items {
            values.push(match item {
                OrderedJson::String(text) => text.clone(),
                OrderedJson::Number(number) => number.to_string(),
                OrderedJson::Bool(flag) => flag.to_string(),
                _ => return None,
            });
        }
        combinations = combinations.saturating_mul(values.len() as u64);
        if combinations > 256 {
            return None;
        }
        axes.insert(axis.clone(), values);
    }
    Some(axes)
}
