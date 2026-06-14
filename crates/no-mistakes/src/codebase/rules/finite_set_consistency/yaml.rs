use std::collections::BTreeSet;

pub(super) fn extract_yaml_sequence(source: &str, key: &str) -> BTreeSet<String> {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return BTreeSet::new();
    };
    let Some(serde_yaml::Value::Sequence(items)) = value_at_key(&value, key) else {
        return BTreeSet::new();
    };
    items
        .iter()
        .filter_map(|item| match item {
            serde_yaml::Value::String(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

fn value_at_key<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    key.split('.').try_fold(value, |current, part| {
        if let Ok(index) = part.parse::<usize>() {
            match current {
                serde_yaml::Value::Sequence(items) => items.get(index),
                _ => None,
            }
        } else {
            current.get(part)
        }
    })
}
