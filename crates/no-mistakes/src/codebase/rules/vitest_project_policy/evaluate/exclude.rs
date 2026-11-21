use super::*;

pub(super) fn check(
    root: &Path,
    options: &Options,
    test: &Value,
    project: &str,
    output: &mut Diagnostics,
) {
    if !options.checks_exclude_globs() {
        return;
    }
    let Some(exclude) = test.property("exclude") else {
        return;
    };
    match missing_globs(&exclude, &options.required_exclude_globs) {
        None => output
            .incomplete
            .push(incomplete(root, &exclude, "test.exclude", project)),
        Some(missing) if missing.is_empty() => {}
        Some(missing) => output
            .findings
            .push(exclude_finding(root, &exclude, project, &missing)),
    }
}

fn missing_globs(exclude: &Value, required: &[String]) -> Option<Vec<String>> {
    if let Kind::OneOf(options) = &exclude.kind {
        let mut lacked = BTreeSet::new();
        let mut unknown = false;
        for option in options {
            match missing_globs(option, required) {
                Some(items) => lacked.extend(items),
                None => unknown = true,
            }
        }
        if unknown && lacked.is_empty() {
            return None;
        }
        return Some(
            required
                .iter()
                .filter(|glob| lacked.contains(*glob))
                .cloned()
                .collect(),
        );
    }
    let Kind::Array(elements) = &exclude.kind else {
        return if matches!(exclude.kind, Kind::Unknown) {
            None
        } else {
            Some(absent(required, &BTreeSet::new()))
        };
    };
    let mut present = BTreeSet::new();
    let mut unknown = false;
    for element in elements {
        match element.literal().and_then(|value| value.as_str()) {
            Some(text) => {
                present.insert(text);
            }
            None => unknown = true,
        }
    }
    let missing = absent(required, &present);
    if missing.is_empty() || !unknown {
        Some(missing)
    } else {
        None
    }
}

fn absent(required: &[String], present: &BTreeSet<&str>) -> Vec<String> {
    let mut missing = Vec::new();
    for glob in required {
        if !present.contains(glob.as_str()) && !missing.iter().any(|seen| seen == glob) {
            missing.push(glob.clone());
        }
    }
    missing
}
