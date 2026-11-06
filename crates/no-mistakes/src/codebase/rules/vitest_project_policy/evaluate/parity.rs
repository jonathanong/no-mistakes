use super::*;
type Baseline = Option<[serde_json::Value; 2]>;

pub(super) struct Project<'a> {
    pub(super) root: &'a Path,
    pub(super) root_test: &'a Value,
    pub(super) object: &'a Value,
    pub(super) test: &'a Value,
    pub(super) name: &'a str,
}

pub(super) fn check(
    project: &Project<'_>,
    pair: &Parity,
    baseline: &mut Baseline,
    output: &mut Diagnostics,
) {
    let Project {
        root,
        root_test,
        object: project,
        test,
        name,
    } = project;
    let [left, right] = &pair.properties;
    let values = [test.property(left), test.property(right)];
    if unknown_values(root, pair, &values, name, output) {
        return;
    }
    match values {
        [None, None] => {}
        [Some(value), None] | [None, Some(value)] => output.findings.push(finding(
            root,
            project,
            name,
            format!(
                "test.{left} and test.{right} must be declared together (found {})",
                value.literal().unwrap()
            ),
        )),
        [Some(a), Some(b)] => {
            let values = [a.literal().unwrap().clone(), b.literal().unwrap().clone()];
            if pair.compare_root {
                compare_root(root, pair, &values, [&a, &b], root_test, name, output);
            } else if let Some(expected) = baseline {
                for (((key, actual), expected), location) in pair
                    .properties
                    .iter()
                    .zip(&values)
                    .zip(expected)
                    .zip([&a, &b])
                {
                    if actual != &*expected {
                        output.findings.push(finding(root, location, name,
                            format!("test.{key} must equal {expected} to match the first complete project")));
                    }
                }
            } else {
                *baseline = Some(values);
            }
        }
    }
}
fn unknown_values(
    root: &Path,
    pair: &Parity,
    values: &[Option<Value>; 2],
    name: &str,
    output: &mut Diagnostics,
) -> bool {
    let mut unknown = false;
    for (key, value) in pair.properties.iter().zip(values) {
        if let Some(value) = value {
            if value.literal().is_none() {
                output.incomplete.push(incomplete(root, value, key, name));
                unknown = true;
            }
        }
    }
    unknown
}
fn compare_root(
    root: &Path,
    pair: &Parity,
    values: &[serde_json::Value; 2],
    locations: [&Value; 2],
    root_test: &Value,
    name: &str,
    output: &mut Diagnostics,
) {
    for ((key, actual), location) in pair.properties.iter().zip(values).zip(locations) {
        match root_test.property(key) {
            Some(expected) if expected.literal().is_none() => output
                .incomplete
                .push(incomplete(root, &expected, key, "<root>")),
            Some(expected) if expected.literal() == Some(actual) => {}
            Some(_) => output.findings.push(finding(
                root,
                location,
                name,
                format!("test.{key} must match the root test.{key}"),
            )),
            None => output
                .incomplete
                .push(incomplete(root, root_test, key, "<root>")),
        }
    }
}
