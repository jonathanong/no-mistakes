use super::*;
mod assertions;
mod parity;

pub(super) fn check(root: &Path, options: &Options, config: Value) -> Diagnostics {
    let mut output = Diagnostics::default();
    let Some(root_test) = config.property("test") else {
        output
            .incomplete
            .push(incomplete(root, &config, "test.projects", "<root>"));
        return output;
    };
    let Some(projects) = root_test.property("projects") else {
        return output;
    };
    let Kind::Array(projects) = &projects.kind else {
        output
            .incomplete
            .push(incomplete(root, &projects, "test.projects", "<root>"));
        return output;
    };
    let mut baselines = vec![None; options.parity.len()];
    for (index, project) in projects.iter().enumerate() {
        let test = project_test(root, project, &root_test, index, &mut output);
        let name = test
            .property("name")
            .and_then(|value| {
                value
                    .literal()
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| format!("#{}", index + 1));
        if !matches!(test.kind, Kind::Object(_, _)) {
            output
                .incomplete
                .push(incomplete(root, project, "test", &name));
            continue;
        }
        assertions::check(root, options, project, &test, &name, &mut output);
        for (pair, baseline) in options.parity.iter().zip(&mut baselines) {
            parity::check(
                &parity::Project {
                    root,
                    root_test: &root_test,
                    object: project,
                    test: &test,
                    name: &name,
                },
                pair,
                baseline,
                &mut output,
            );
        }
    }
    output
}

fn project_test(
    root: &Path,
    project: &Value,
    root_test: &Value,
    index: usize,
    output: &mut Diagnostics,
) -> Value {
    let mut test = project.property("test").unwrap_or_else(|| Value {
        kind: Kind::Object(BTreeMap::new(), false),
        ..project.clone()
    });
    match project.property("extends") {
        Some(value) if value.literal() == Some(&true.into()) => test = inherit(root_test, &test),
        Some(value) if value.literal() != Some(&false.into()) => {
            output.incomplete.push(incomplete(
                root,
                project,
                "extends",
                &format!("#{}", index + 1),
            ));
            if let Kind::Object(fields, _) = &test.kind {
                test.kind = Kind::Object(fields.clone(), true);
            }
        }
        _ => {}
    }
    test
}
fn inherit(root: &Value, project: &Value) -> Value {
    match (&root.kind, &project.kind) {
        (Kind::Object(base, base_unknown), Kind::Object(fields, unknown)) => {
            let mut merged = if *unknown {
                BTreeMap::new()
            } else {
                base.clone()
            };
            merged.extend(fields.clone());
            Value {
                kind: Kind::Object(merged, *base_unknown || *unknown),
                ..project.clone()
            }
        }
        _ => project.unknown(),
    }
}
