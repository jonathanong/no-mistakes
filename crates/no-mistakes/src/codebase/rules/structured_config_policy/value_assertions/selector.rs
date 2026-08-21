use serde_yaml::Value;

pub(crate) struct SelectorValues<'a> {
    pub(crate) values: Vec<&'a Value>,
    pub(crate) has_missing: bool,
}

pub(crate) fn values_at_selector<'a>(value: &'a Value, selector: &str) -> SelectorValues<'a> {
    walk(value, selector, false)
}

pub(crate) fn any_groups<'a>(value: &'a Value, selector: &str) -> Vec<Vec<&'a Value>> {
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
            groups.push(Vec::new());
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
        for selected in current {
            step_part(selected, part, skip_missing, &mut next, &mut has_missing);
        }
        current = next;
    }
    SelectorValues {
        values: current.into_iter().flatten().collect(),
        has_missing,
    }
}

fn step_part<'a>(
    selected: Option<&'a Value>,
    part: &str,
    skip_missing: bool,
    next: &mut Vec<Option<&'a Value>>,
    has_missing: &mut bool,
) {
    if part == "[]" {
        if let Some(Value::Sequence(items)) = selected {
            next.extend(items.iter().map(Some));
            return;
        }
        push_missing(skip_missing, next, has_missing);
        return;
    }
    if let Ok(index) = part.parse::<usize>() {
        if let Some(Value::Sequence(items)) = selected {
            match items.get(index) {
                Some(item) => next.push(Some(item)),
                None => push_missing(skip_missing, next, has_missing),
            }
            return;
        }
        push_missing(skip_missing, next, has_missing);
        return;
    }
    match selected.and_then(|selected| selected.get(part)) {
        Some(child) => next.push(Some(child)),
        None => push_missing(skip_missing, next, has_missing),
    }
}

fn push_missing(skip_missing: bool, next: &mut Vec<Option<&Value>>, has_missing: &mut bool) {
    if skip_missing {
        return;
    }
    *has_missing = true;
    next.push(None);
}
