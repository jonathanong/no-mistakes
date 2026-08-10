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
    if axes.is_empty() {
        return true;
    }
    let exclusions: &[Value] = match matrix.get("exclude") {
        Some(Value::Sequence(exclusions)) if exclusions.iter().all(Value::is_mapping) => exclusions,
        Some(_) => return false,
        None => &[],
    };
    !has_unexcluded_combination(&axes, exclusions, 0, &mut BTreeMap::new())
}

pub(crate) fn matrix_shape_valid(job: &Value) -> bool {
    let Some(matrix) = job
        .get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
        .and_then(Value::as_mapping)
    else {
        return true;
    };
    match static_matrix_job_count(matrix) {
        Some(count) => count <= 256,
        None => true,
    }
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
    Some(axes)
}

fn static_matrix_job_count(mapping: &serde_yaml::Mapping) -> Option<usize> {
    let axes = static_matrix_axes(mapping)?;
    let exclusions = sequence_of_mappings(mapping.get("exclude"))?;
    let includes = sequence_of_mappings(mapping.get("include"))?;
    let mut values = BTreeMap::new();
    let mut count = count_unexcluded(&axes, &exclusions, 0, &mut values, 257);
    for include in includes {
        values.clear();
        if !has_applicable_combination(&axes, &exclusions, include, 0, &mut values) {
            count = count.saturating_add(1).min(257);
        }
    }
    Some(count)
}

fn sequence_of_mappings(value: Option<&Value>) -> Option<Vec<&serde_yaml::Mapping>> {
    match value {
        Some(Value::Sequence(items)) => items.iter().map(Value::as_mapping).collect(),
        Some(_) => None,
        None => Some(Vec::new()),
    }
}

fn count_unexcluded(
    axes: &[(String, Vec<Value>)],
    exclusions: &[&serde_yaml::Mapping],
    index: usize,
    values: &mut BTreeMap<String, Value>,
    limit: usize,
) -> usize {
    if exclusions
        .iter()
        .any(|exclusion| exclusion_matches_assigned(exclusion, values))
    {
        return 0;
    }
    let Some((name, choices)) = axes.get(index) else {
        return usize::from(!axes.is_empty());
    };
    let mut count = 0_usize;
    for choice in choices {
        values.insert(name.clone(), choice.clone());
        count += count_unexcluded(axes, exclusions, index + 1, values, limit - count);
        if count >= limit {
            break;
        }
    }
    values.remove(name);
    count
}

fn exclusion_matches_assigned(
    exclusion: &serde_yaml::Mapping,
    values: &BTreeMap<String, Value>,
) -> bool {
    exclusion.iter().all(|(name, value)| {
        name.as_str()
            .and_then(|name| values.get(name))
            .is_some_and(|actual| actual == value)
    })
}

fn has_applicable_combination(
    axes: &[(String, Vec<Value>)],
    exclusions: &[&serde_yaml::Mapping],
    include: &serde_yaml::Mapping,
    index: usize,
    values: &mut BTreeMap<String, Value>,
) -> bool {
    if exclusions
        .iter()
        .any(|exclusion| exclusion_matches_assigned(exclusion, values))
    {
        return false;
    }
    let Some((name, choices)) = axes.get(index) else {
        return !axes.is_empty();
    };
    let included_value = include.get(name);
    for choice in choices {
        if included_value.is_some_and(|included| included != choice) {
            continue;
        }
        values.insert(name.clone(), choice.clone());
        let applicable = has_applicable_combination(axes, exclusions, include, index + 1, values);
        values.remove(name);
        if applicable {
            return true;
        }
    }
    false
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
