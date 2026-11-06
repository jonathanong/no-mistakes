use super::*;

pub(super) fn check(
    root: &Path,
    options: &Options,
    project: &Value,
    test: &Value,
    name: &str,
    output: &mut Diagnostics,
) {
    for (property, required) in &options.required {
        match test.property(property) {
            Some(value) if value.literal().is_none() => output
                .incomplete
                .push(incomplete(root, &value, property, name)),
            Some(value)
                if value
                    .literal()
                    .is_some_and(|actual| literals_equal(actual, required)) => {}
            value => output.findings.push(finding(
                root,
                value.as_ref().unwrap_or(project),
                name,
                format!("test.{property} must equal {required}"),
            )),
        }
    }
    for property in &options.forbidden {
        if let Some(value) = test.property(property) {
            if matches!(&test.kind, Kind::Object(fields, true) if !fields.contains_key(property)) {
                output
                    .incomplete
                    .push(incomplete(root, &value, property, name));
            } else {
                output.findings.push(finding(
                    root,
                    &value,
                    name,
                    format!("test.{property} is forbidden"),
                ));
            }
        }
    }
}

fn literals_equal(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual.as_number(), expected.as_number()) {
        (Some(actual), Some(expected)) => actual.as_f64() == expected.as_f64(),
        _ => actual == expected,
    }
}
