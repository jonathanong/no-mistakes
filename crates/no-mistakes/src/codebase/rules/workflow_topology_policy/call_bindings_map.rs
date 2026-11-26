use super::call_bindings_yaml::{describe_value, is_scalar, RawBindingMap, RawWorkflows};
use super::{job_finding, LocatedFinding};
use crate::config::source_locations::ConfigPath;
use serde_yaml::Value;
use std::collections::BTreeMap;

pub(super) struct BindingCheck<'a> {
    pub(super) call_from: &'a str,
    pub(super) caller_path: &'a str,
    pub(super) job_key: &'a str,
    pub(super) callee_path: &'a str,
    pub(super) field: &'a str,
    pub(super) noun: &'a str,
    pub(super) action: &'a str,
    pub(super) declared: &'a [&'a str],
}

pub(super) fn check_declared_map(
    spec: BindingCheck<'_>,
    raw: &RawWorkflows,
    findings: &mut Vec<LocatedFinding>,
    unanalyzable: &mut Vec<LocatedFinding>,
) {
    let provided = match raw.job_field(spec.caller_path, spec.job_key, spec.field) {
        RawBindingMap::Absent | RawBindingMap::Inherit => Vec::new(),
        RawBindingMap::Mapping(entries) => entries,
        RawBindingMap::NonMapping(shape) => {
            unanalyzable.push(unanalyzable_job_field(
                spec.call_from,
                spec.job_key,
                spec.field,
                format!(
                    "reusable workflow call `{}` sets `{}` to `{shape}`, which cannot be verified statically; use a literal mapping",
                    spec.call_from, spec.field
                ),
            ));
            return;
        }
    };
    let mut declared_groups: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for &name in spec.declared {
        declared_groups
            .entry(name.to_lowercase())
            .or_default()
            .push(name);
    }
    let mut provided_groups: BTreeMap<String, Vec<(String, &Value)>> = BTreeMap::new();
    for (key, value) in provided {
        provided_groups
            .entry(key.to_lowercase())
            .or_default()
            .push((key, value));
    }
    let has_field = raw.has_job_field(spec.caller_path, spec.job_key, spec.field);
    for (lower, names) in declared_groups {
        if names.len() > 1 {
            unanalyzable.push(unanalyzable_job_field(
                spec.call_from,
                spec.job_key,
                "uses",
                format!(
                    "reusable workflow `{}` declares case-colliding {}s ({}), which cannot be verified statically",
                    spec.callee_path, spec.noun, names.join(", ")
                ),
            ));
            continue;
        }
        let name = names[0];
        match provided_groups.get(&lower).map(Vec::as_slice) {
            None | Some([]) => {
                let message = format!(
                    "reusable workflow call `{}` does not pass {} `{name}` to `{}`; {} `{name}` explicitly under `{}:`",
                    spec.call_from, spec.noun, spec.callee_path, spec.action, spec.field
                );
                let finding = if has_field {
                    job_field_finding(spec.call_from, spec.job_key, spec.field, message)
                } else {
                    job_finding(spec.call_from, message)
                };
                findings.push(finding);
            }
            Some([(key, value)]) if value.is_null() => {
                let mut finding = job_field_finding(
                    spec.call_from,
                    spec.job_key,
                    spec.field,
                    format!(
                        "reusable workflow call `{}` leaves {} `{key}` empty for `{}`; {} `{name}` explicitly under `{}:`",
                        spec.call_from, spec.noun, spec.callee_path, spec.action, spec.field
                    ),
                );
                finding.path = ConfigPath::default()
                    .key("jobs")
                    .key(spec.job_key)
                    .key(spec.field)
                    .key(key);
                findings.push(finding);
            }
            Some([(key, value)]) if !is_scalar(value) => {
                let mut finding = unanalyzable_job_field(
                    spec.call_from,
                    spec.job_key,
                    spec.field,
                    format!(
                        "reusable workflow call `{}` sets `{}.{key}` to `{}`, which cannot be verified statically",
                        spec.call_from,
                        spec.field,
                        describe_value(value)
                    ),
                );
                finding.path = ConfigPath::default()
                    .key("jobs")
                    .key(spec.job_key)
                    .key(spec.field)
                    .key(key);
                unanalyzable.push(finding);
            }
            Some([_]) => {}
            Some(_) => {
                unanalyzable.push(unanalyzable_job_field(
                    spec.call_from,
                    spec.job_key,
                    spec.field,
                    format!(
                        "reusable workflow call `{}` has duplicate case-colliding `{}` keys for `{name}`, which cannot be verified statically",
                        spec.call_from, spec.field
                    ),
                ));
            }
        }
    }
}

pub(super) fn job_field_finding(
    call_from: &str,
    job_key: &str,
    field: &str,
    message: String,
) -> LocatedFinding {
    let mut finding = job_finding(call_from, message);
    finding.path = ConfigPath::default().key("jobs").key(job_key).key(field);
    finding.fallback_path = Some(ConfigPath::default().key("jobs").key(job_key));
    finding
}

pub(super) fn unanalyzable_job_field(
    call_from: &str,
    job_key: &str,
    field: &str,
    message: String,
) -> LocatedFinding {
    let mut finding = job_field_finding(call_from, job_key, field, message);
    finding.finding.target = Some("unanalyzable".to_owned());
    finding
}
