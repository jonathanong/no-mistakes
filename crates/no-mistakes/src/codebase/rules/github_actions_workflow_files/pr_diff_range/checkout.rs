use super::exprs::extract;
use super::yaml_shapes::get;
use serde_yaml::{Mapping, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CheckoutState {
    MergeCommit,
    PullRequestHead,
    PullRequestBase,
    MergeGroup,
    Unknown,
}

impl CheckoutState {
    pub(super) fn initial(pull: bool, queue: bool) -> Self {
        if pull {
            Self::MergeCommit
        } else if queue {
            Self::MergeGroup
        } else {
            Self::Unknown
        }
    }
}

pub(super) fn next_checkout_state(
    current: CheckoutState,
    step: &Mapping,
    pull: bool,
    queue: bool,
) -> CheckoutState {
    let Some(uses) = get(step, "uses").and_then(Value::as_str) else {
        return current;
    };
    if !is_checkout_action(uses) {
        return current;
    }
    let Some(with_raw) = get(step, "with") else {
        return CheckoutState::initial(pull, queue);
    };
    if with_raw.is_null() {
        return CheckoutState::initial(pull, queue);
    }
    let Some(with_map) = with_raw.as_mapping() else {
        return CheckoutState::Unknown;
    };
    if with_map.contains_key(Value::String("<<".to_owned())) {
        return CheckoutState::Unknown;
    }
    let Some(ref_val) = get(with_map, "ref") else {
        return CheckoutState::initial(pull, queue);
    };
    if ref_val.is_null() {
        return CheckoutState::initial(pull, queue);
    }
    let Some(ref_text) = ref_val.as_str() else {
        return CheckoutState::Unknown;
    };
    let trimmed = ref_text.trim();
    if trimmed.is_empty() {
        return CheckoutState::initial(pull, queue);
    }
    classify_checkout_ref(trimmed, pull, queue)
}

fn classify_checkout_ref(text: &str, pull: bool, queue: bool) -> CheckoutState {
    let (replaced, exprs) = extract(text);
    if let Some(expr) = single_expr(&replaced, &exprs) {
        return match expr {
            "github.sha" => CheckoutState::initial(pull, queue),
            "github.event.pull_request.head.sha"
            | "github.head_ref"
            | "github.event.pull_request.head.ref" => CheckoutState::PullRequestHead,
            "github.event.pull_request.base.sha"
            | "github.base_ref"
            | "github.event.pull_request.base.ref" => CheckoutState::PullRequestBase,
            "github.event.merge_group.head_sha" => CheckoutState::MergeGroup,
            _ => CheckoutState::Unknown,
        };
    }
    if replaced == "refs/pull/__ghx0__/merge"
        && exprs.first().map(String::as_str) == Some("github.event.pull_request.number")
    {
        return CheckoutState::MergeCommit;
    }
    CheckoutState::Unknown
}

fn single_expr<'a>(replaced: &str, exprs: &'a [String]) -> Option<&'a str> {
    let index = replaced
        .strip_prefix("__ghx")?
        .strip_suffix("__")?
        .parse::<usize>()
        .ok()?;
    exprs.get(index).map(String::as_str)
}

pub(super) fn job_effective_checkout(
    steps: Option<&Vec<Value>>,
    pull: bool,
    queue: bool,
) -> CheckoutState {
    let state = CheckoutState::initial(pull, queue);
    for step in steps.into_iter().flatten() {
        let Some(step_map) = step.as_mapping() else {
            continue;
        };
        let next = next_checkout_state(state, step_map, pull, queue);
        if next != state || get(step_map, "uses").and_then(Value::as_str).is_some_and(is_checkout_action) {
            return next;
        }
    }
    state
}

fn is_checkout_action(uses: &str) -> bool {
    let name = uses.trim().split('@').next().unwrap_or("");
    let name = name.strip_suffix('/').unwrap_or(name);
    name.eq_ignore_ascii_case("actions/checkout")
}
