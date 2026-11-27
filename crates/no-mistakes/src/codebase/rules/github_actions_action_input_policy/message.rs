use super::policy::InputFault;
use super::RULE_ID;
use no_mistakes_rule_kernel::finding::RuleFinding;

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
        | InputFault::Banned { key }
        | InputFault::Unlisted { key }
        | InputFault::NotPresent { key } => key,
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
        InputFault::Unlisted { key } if key == "<<" => format!(
            "{where_} with a YAML merge key `<<` under `with:`. The allowlist cannot check merged inputs. Write the inputs inline under `with:` so the policy can check each one."
        ),
        InputFault::Unlisted { key } => format!(
            "{where_} with input `{key}`, which is not in `allowedInputs`. Remove `{key}` from `with:`, or add it to `allowedInputs` if the action may receive it. The action policy lists every input this action may receive."
        ),
        InputFault::NotPresent { key } => format!(
            "{where_} without required input `{key}`. Set `{key}` to a non-null value. The action policy requires this input on every ref."
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
