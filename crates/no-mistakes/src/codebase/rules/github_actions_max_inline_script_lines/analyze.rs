use super::{finding, step_script_lines, RuleFinding, StepScript};
use serde_yaml::Value;

pub(super) fn check_source(rel: &str, source: &str, action: bool, max: usize) -> Vec<RuleFinding> {
    let source = source.trim_start_matches('\u{feff}');
    let value: Value = match serde_yaml::from_str(source) {
        Ok(value) => value,
        Err(error) => {
            return vec![finding(
                rel,
                error.location().map_or(1, |loc| loc.line()),
                format!(
                    "Invalid YAML ({error}); fix the YAML so inline script length can be checked."
                ),
            )]
        }
    };
    if action && value["runs"]["using"].as_str() != Some("composite") {
        return Vec::new();
    }
    let mut expected = Vec::new();
    if action {
        collect_steps(&value["runs"]["steps"], &mut expected);
    } else if let Some(jobs) = value["jobs"].as_mapping() {
        for job in jobs.values() {
            collect_steps(&job["steps"], &mut expected);
        }
    }
    let scripts = step_script_lines(source, action);
    if expected.len() != scripts.len()
        || expected
            .iter()
            .zip(&scripts)
            .any(|(kind, script)| *kind != script.github_script)
    {
        return vec![finding(rel, 1, "Script bodies could not be aligned to source locations; rewrite anchors, aliases, and flow-style steps as ordinary block-style run or with.script bodies so inline script length can be checked.".into())];
    }
    scripts.into_iter().filter(|script| script.count > max).map(|StepScript { line, count, .. }|
        finding(rel, line, format!("Inline script has {count} non-blank lines, exceeding maxLines {max}; move it into a file and call that file so it can be linted and tested independently."))
    ).collect()
}

fn collect_steps(steps: &Value, expected: &mut Vec<bool>) {
    let Some(steps) = steps.as_sequence() else {
        return;
    };
    for step in steps {
        if step["run"].as_str().is_some() {
            expected.push(false);
        }
        if step["uses"].as_str().is_some_and(|uses| {
            uses.split('@')
                .next()
                .unwrap_or(uses)
                .trim_end_matches('/')
                .eq_ignore_ascii_case("actions/github-script")
        }) && step["with"]["script"].as_str().is_some()
        {
            expected.push(true);
        }
    }
}
