use super::OrderedJson;
use std::collections::BTreeMap;

pub(super) type SimpleMatrixAxes<'a> = BTreeMap<&'a str, &'a [OrderedJson]>;

/// Borrow and validate a simple matrix without copying axis names or scalar
/// values. The combination cap is checked against raw entries before duplicate
/// names overwrite earlier values, matching the original axis projection.
pub(super) fn simple_matrix_axes(matrix: Option<&OrderedJson>) -> Option<SimpleMatrixAxes<'_>> {
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
        if items.is_empty()
            || items.iter().any(|item| {
                !matches!(
                    item,
                    OrderedJson::String(_) | OrderedJson::Number(_) | OrderedJson::Bool(_)
                )
            })
        {
            return None;
        }
        combinations = combinations.saturating_mul(items.len() as u64);
        if combinations > 256 {
            return None;
        }
        axes.insert(axis.as_str(), items.as_slice());
    }
    Some(axes)
}

pub(super) fn matrix_combination_count(axes: &SimpleMatrixAxes<'_>) -> u32 {
    axes.values().map(|items| items.len() as u32).product()
}

/// Convert validated values only for the dynamic expansion path, which needs
/// owned replacement strings for each matrix combination.
pub(super) fn matrix_axis_values<'a>(
    axes: SimpleMatrixAxes<'a>,
) -> Option<BTreeMap<&'a str, Vec<String>>> {
    let mut values_by_axis = BTreeMap::new();
    for (axis, items) in axes {
        let values = items
            .iter()
            .map(matrix_axis_value)
            .collect::<Option<Vec<_>>>()?;
        values_by_axis.insert(axis, values);
    }
    Some(values_by_axis)
}

fn matrix_axis_value(item: &OrderedJson) -> Option<String> {
    match item {
        OrderedJson::String(text) => Some(text.clone()),
        OrderedJson::Number(number) => Some(number.to_string()),
        OrderedJson::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{matrix_axis_values, SimpleMatrixAxes};
    use crate::codebase::workflow_topology::value_primitives::OrderedJson;
    use std::collections::BTreeMap;

    #[test]
    fn matrix_axis_values_formats_each_supported_scalar() {
        let items = [
            OrderedJson::String("linux".to_string()),
            OrderedJson::Number(22.into()),
            OrderedJson::Bool(true),
        ];
        let axes: SimpleMatrixAxes<'_> = BTreeMap::from([("axis", &items[..])]);
        assert_eq!(
            matrix_axis_values(axes).unwrap()["axis"],
            vec!["linux", "22", "true"]
        );
    }

    #[test]
    fn matrix_axis_values_rejects_non_scalar_values() {
        let items = [OrderedJson::Null];
        let axes: SimpleMatrixAxes<'_> = BTreeMap::from([("axis", &items[..])]);
        assert!(matrix_axis_values(axes).is_none());
    }
}
