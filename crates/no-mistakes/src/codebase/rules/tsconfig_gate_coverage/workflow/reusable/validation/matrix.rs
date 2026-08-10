use serde_yaml::Value;
use std::collections::BTreeMap;

pub(crate) fn zero_instance_matrix(job: &Value) -> bool {
    let Some(matrix) = job
        .get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
        .and_then(Value::as_mapping)
    else {
        return false;
    };
    let includes_instance = match matrix.get("include") {
        Some(Value::Sequence(items)) => !items.is_empty(),
        Some(_) => return false,
        None => false,
    };
    if includes_instance {
        return false;
    }
    let Some(axes) = static_matrix_axes(matrix) else {
        return false;
    };
    let exclusions: &[Value] = match matrix.get("exclude") {
        Some(Value::Sequence(exclusions)) if exclusions.iter().all(Value::is_mapping) => exclusions,
        Some(_) => return false,
        None => &[],
    };
    !has_unexcluded_combination(&axes, exclusions, 0, &mut BTreeMap::new())
}

fn static_matrix_axes(mapping: &serde_yaml::Mapping) -> Option<Vec<(String, Vec<Value>)>> {
    let axes = mapping
        .iter()
        .filter(|(name, _)| !matches!(name.as_str(), Some("include" | "exclude")))
        .map(|(name, values)| {
            let name = name.as_str()?;
            let values = values.as_sequence()?;
            values
                .iter()
                .all(|value| !matches!(value, Value::Mapping(_) | Value::Sequence(_)))
                .then(|| (name.to_string(), values.clone()))
        })
        .collect::<Option<Vec<_>>>()?;
    axes.iter().try_fold(1_usize, |count, (_, values)| {
        count
            .checked_mul(values.len())
            .filter(|count| *count <= 256)
    })?;
    (!axes.is_empty()).then_some(axes)
}

fn has_unexcluded_combination(
    axes: &[(String, Vec<Value>)],
    exclusions: &[Value],
    index: usize,
    values: &mut BTreeMap<String, Value>,
) -> bool {
    let Some((name, choices)) = axes.get(index) else {
        return !exclusions.iter().any(|exclusion| {
            exclusion.as_mapping().is_some_and(|fields| {
                fields.iter().all(|(name, value)| {
                    name.as_str()
                        .and_then(|name| values.get(name))
                        .is_some_and(|actual| actual == value)
                })
            })
        });
    };
    choices.iter().any(|choice| {
        values.insert(name.clone(), choice.clone());
        has_unexcluded_combination(axes, exclusions, index + 1, values)
    })
}

#[cfg(test)]
mod tests;
