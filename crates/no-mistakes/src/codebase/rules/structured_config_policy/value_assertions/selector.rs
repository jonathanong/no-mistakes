use serde_yaml::Value;

pub(super) struct SelectorValues<'a> {
    pub(super) values: Vec<&'a Value>,
    pub(super) has_missing: bool,
}

pub(super) fn values_at_selector<'a>(value: &'a Value, selector: &str) -> SelectorValues<'a> {
    walk(value, selector, false)
}

pub(super) fn any_groups<'a>(value: &'a Value, selector: &str) -> Vec<Vec<&'a Value>> {
    let parts: Vec<&str> = selector
        .split('.')
        .filter(|part| !part.is_empty())
        .collect();
    let Some(last) = parts.iter().rposition(|part| *part == "[]") else {
        return walk(value, selector, true)
            .values
            .into_iter()
            .map(|selected| vec![selected])
            .collect();
    };
    let parent_sel = parts[..last].join(".");
    let rest = parts[last + 1..].join(".");
    let parents = if parent_sel.is_empty() {
        vec![value]
    } else {
        walk(value, &parent_sel, true).values
    };
    let mut groups = Vec::new();
    for parent in parents {
        let Some(items) = parent.as_sequence() else {
            continue;
        };
        let mut group = Vec::new();
        for item in items {
            if rest.is_empty() {
                group.push(item);
            } else {
                group.extend(walk(item, &rest, true).values);
            }
        }
        groups.push(group);
    }
    groups
}

fn walk<'a>(value: &'a Value, selector: &str, skip_missing: bool) -> SelectorValues<'a> {
    let mut current = vec![Some(value)];
    let mut has_missing = false;
    for part in selector.split('.').filter(|part| !part.is_empty()) {
        let mut next = Vec::new();
        if part == "[]" {
            for selected in current {
                match selected {
                    Some(Value::Sequence(items)) => next.extend(items.iter().map(Some)),
                    Some(_) | None if skip_missing => {}
                    Some(_) | None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        } else if let Ok(index) = part.parse::<usize>() {
            for selected in current {
                match selected {
                    Some(Value::Sequence(items)) => match items.get(index) {
                        Some(item) => next.push(Some(item)),
                        None if skip_missing => {}
                        None => {
                            has_missing = true;
                            next.push(None);
                        }
                    },
                    Some(_) | None if skip_missing => {}
                    Some(_) | None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        } else {
            for selected in current {
                match selected.and_then(|selected| selected.get(part)) {
                    Some(child) => next.push(Some(child)),
                    None if skip_missing => {}
                    None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        }
        current = next;
    }
    SelectorValues {
        values: current.into_iter().flatten().collect(),
        has_missing,
    }
}
