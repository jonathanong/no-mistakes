use super::policy::InputFault;
use super::{RuleFinding, RULE_ID};

#[derive(Clone, Copy)]
pub(super) enum Site<'a> {
    Step { job_id: &'a str, index: usize },
    Composite { index: usize },
    Job { job_id: &'a str },
}

pub(super) fn fault_key(fault: &InputFault) -> &str {
    match fault {
        InputFault::Missing { key, .. }
        | InputFault::Mismatch { key, .. }
        | InputFault::Banned { key } => key,
    }
}

pub(super) fn message(
    rel: &str,
    site: Site<'_>,
    label: &str,
    family: &str,
    fault: &InputFault,
) -> String {
    let where_ = match site {
        Site::Step { job_id, .. } => {
            format!("{rel}: job \"{job_id}\" step \"{label}\" calls `{family}`")
        }
        Site::Composite { .. } => format!("{rel}: composite step \"{label}\" calls `{family}`"),
        Site::Job { job_id } => format!("{rel}: job \"{job_id}\" calls `{family}`"),
    };
    match fault {
        InputFault::Missing { key, expected } => format!(
            "{where_} without required input `{key}`. Set `{key}` to {expected}. The action policy requires this value on every ref."
        ),
        InputFault::Mismatch { key, expected, got } => format!(
            "{where_} with `{key}` set to {got}. Set `{key}` to {expected}. The action policy requires this value on every ref."
        ),
        InputFault::Banned { key } => format!(
            "{where_} and sets banned input `{key}`. Remove `{key}` from `with:`."
        ),
    }
}

pub(super) fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
