use super::{finding, step_script_lines, RuleFinding, ScriptScan, StepScript};
use crate::fx::FxHashMap;
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
        collect_steps("", &value["runs"]["steps"], &mut expected);
    } else if let Some(jobs) = value["jobs"].as_mapping() {
        for (name, job) in jobs {
            collect_steps(
                name.as_str().unwrap_or_default(),
                &job["steps"],
                &mut expected,
            );
        }
    }
    let ScriptScan {
        scripts,
        step_lines,
    } = step_script_lines(source, action);
    let mut scripts: FxHashMap<_, _> = scripts
        .into_iter()
        .map(|script| {
            (
                (script.job.clone(), script.step, script.github_script),
                script,
            )
        })
        .collect();
    let mut aligned = Vec::new();
    let mut findings = Vec::new();
    for key in expected {
        match scripts.remove(&key) {
            Some(script) if script.supported => aligned.push(script),
            script => {
                let fallback = step_lines.get(&(key.0, key.1)).copied().unwrap_or(1);
                findings.push(unaligned(
                    rel,
                    script.map_or(fallback, |script| script.line),
                ));
            }
        }
    }
    findings.extend(
        scripts
            .into_values()
            .map(|script| unaligned(rel, script.line)),
    );
    let context = fix_context(action, &value["on"]);
    findings.extend(aligned.into_iter().filter(|script| script.count > max).map(|StepScript { line, count, .. }|
        finding(rel, line, format!("Inline script has {count} non-blank lines, exceeding maxLines {max}; move it into a file and call that file so it can be linted and tested independently.{context}"))
    ));
    findings
}

fn unaligned(rel: &str, line: usize) -> RuleFinding {
    finding(rel, line, "Script body could not be aligned to its source location; rewrite anchors, aliases, and flow-style steps as an ordinary block-style run or with.script body so inline script length can be checked.".into())
}

fn collect_steps(job: &str, steps: &Value, expected: &mut Vec<(String, usize, bool)>) {
    let Some(steps) = steps.as_sequence() else {
        return;
    };
    for (index, step) in steps.iter().enumerate() {
        if step["run"].as_str().is_some() {
            expected.push((job.into(), index + 1, false));
        }
        if step["uses"].as_str().is_some_and(|uses| {
            uses.split('@')
                .next()
                .unwrap_or(uses)
                .trim()
                .trim_end_matches('/')
                .eq_ignore_ascii_case("actions/github-script")
        }) && step["with"]["script"].as_str().is_some()
        {
            expected.push((job.into(), index + 1, true));
        }
    }
}

fn fix_context(action: bool, triggers: &Value) -> &'static str {
    if action {
        return " Use $GITHUB_ACTION_PATH for a script beside the action; a repo-relative path depends on the caller's checkout.";
    }
    if ["workflow_run", "pull_request_target"]
        .iter()
        .any(|event| has_event(triggers, event))
    {
        return " The moved script must come from the trusted ref, never the PR head.";
    }
    ""
}

fn has_event(triggers: &Value, event: &str) -> bool {
    triggers.as_str() == Some(event)
        || triggers
            .as_sequence()
            .is_some_and(|events| events.iter().any(|value| value.as_str() == Some(event)))
        || triggers
            .as_mapping()
            .is_some_and(|events| events.contains_key(Value::String(event.into())))
}
