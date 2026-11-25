use super::lines::Seg;
use super::scalar_text;
use serde_yaml::{Mapping, Value};

pub(in crate::codebase::rules) enum ScopeEvent<'a> {
    Mapping {
        owner: &'a Mapping,
        path: &'a [Seg],
        scope: &'a str,
    },
    Unanalyzable {
        path: &'a [Seg],
        target: &'static str,
        description: String,
    },
}

pub(in crate::codebase::rules) fn shape_detail(value: &Value, fallback: &str) -> String {
    if value.is_null() {
        "null".into()
    } else {
        scalar_text(value).unwrap_or_else(|| fallback.into())
    }
}

pub(in crate::codebase::rules) fn visit_workflow_scopes(
    value: &Value,
    visit: &mut impl FnMut(ScopeEvent<'_>),
) {
    let Some(top) = value.as_mapping() else {
        visit(ScopeEvent::Unanalyzable {
            path: &[],
            target: "workflow",
            description: format!(
                "workflow document is `{}` instead of a mapping",
                shape_detail(value, "a non-mapping value")
            ),
        });
        return;
    };
    visit_mapping(top, &[], "workflow", visit);
    let Some(raw_jobs) = top.get("jobs") else {
        return;
    };
    let Some(jobs) = raw_jobs.as_mapping() else {
        let at = [Seg::Key("jobs".into())];
        visit(ScopeEvent::Unanalyzable {
            path: &at,
            target: "jobs",
            description: format!(
                "workflow sets `jobs` to `{}` instead of a mapping",
                shape_detail(raw_jobs, "a non-mapping value")
            ),
        });
        return;
    };
    for (key, raw_job) in jobs {
        let Some(id) = key.as_str() else {
            let at = [Seg::Key("jobs".into())];
            visit(ScopeEvent::Unanalyzable {
                path: &at,
                target: "jobs",
                description: format!(
                    "workflow `jobs` has non-string key `{}`",
                    shape_detail(key, "a non-scalar key")
                ),
            });
            continue;
        };
        let base = [Seg::Key("jobs".into()), Seg::Key(id.into())];
        if id == "<<" {
            visit(ScopeEvent::Unanalyzable {
                path: &base,
                target: "jobs",
                description: "workflow `jobs` uses YAML merge key `<<`".into(),
            });
            continue;
        }
        let owner = format!("job \"{id}\"");
        let Some(job) = raw_job.as_mapping() else {
            visit(ScopeEvent::Unanalyzable {
                path: &base,
                target: "jobs",
                description: format!(
                    "{owner} is `{}` instead of a mapping",
                    shape_detail(raw_job, "a non-mapping value")
                ),
            });
            continue;
        };
        visit_mapping(job, &base, &owner, visit);
        if let Some(raw_steps) = job.get("steps") {
            let at = [base[0].clone(), base[1].clone(), Seg::Key("steps".into())];
            visit_step_scopes(raw_steps, &at, &owner, visit);
        }
    }
}

pub(super) fn visit_step_scopes(
    raw_steps: &Value,
    path: &[Seg],
    owner: &str,
    visit: &mut impl FnMut(ScopeEvent<'_>),
) {
    let Some(steps) = raw_steps.as_sequence() else {
        visit(ScopeEvent::Unanalyzable {
            path,
            target: "steps",
            description: format!(
                "{owner} sets `steps` to `{}` instead of a sequence",
                shape_detail(raw_steps, "a non-sequence value")
            ),
        });
        return;
    };
    for (index, step) in steps.iter().enumerate() {
        let mut at = path.to_vec();
        at.push(Seg::Idx(index));
        let scope = format!("{owner} step {}", index + 1);
        let Some(step_map) = step.as_mapping() else {
            visit(ScopeEvent::Unanalyzable {
                path: &at,
                target: "steps",
                description: format!(
                    "{scope} is `{}` instead of a mapping",
                    shape_detail(step, "a non-mapping value")
                ),
            });
            continue;
        };
        visit_mapping(step_map, &at, &scope, visit);
    }
}

pub(super) fn visit_action_scopes(value: &Value, visit: &mut impl FnMut(ScopeEvent<'_>)) {
    let Some(raw_runs) = value.as_mapping().and_then(|top| top.get("runs")) else {
        return;
    };
    match raw_runs.as_mapping() {
        Some(runs) => {
            if runs.contains_key("<<") {
                let at = [Seg::Key("runs".into()), Seg::Key("<<".into())];
                visit(ScopeEvent::Unanalyzable {
                    path: &at,
                    target: "runs",
                    description: "action `runs` uses YAML merge key `<<`".into(),
                });
            }
            if runs.get("using").and_then(Value::as_str) == Some("composite") {
                if let Some(raw_steps) = runs.get("steps") {
                    let at = [Seg::Key("runs".into()), Seg::Key("steps".into())];
                    visit_step_scopes(raw_steps, &at, "composite action", visit);
                }
            }
        }
        None => {
            let at = [Seg::Key("runs".into())];
            visit(ScopeEvent::Unanalyzable {
                path: &at,
                target: "runs",
                description: format!(
                    "action sets `runs` to `{}` instead of a mapping",
                    shape_detail(raw_runs, "a non-mapping value")
                ),
            });
        }
    }
}

fn visit_mapping(
    owner: &Mapping,
    path: &[Seg],
    scope: &str,
    visit: &mut impl FnMut(ScopeEvent<'_>),
) {
    if owner.contains_key("<<") {
        let mut at = path.to_vec();
        at.push(Seg::Key("<<".into()));
        visit(ScopeEvent::Unanalyzable {
            path: &at,
            target: "merge",
            description: format!("{scope} uses YAML merge key `<<`"),
        });
    }
    visit(ScopeEvent::Mapping { owner, path, scope });
}
