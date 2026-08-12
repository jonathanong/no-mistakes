use super::checkout::CheckoutState;
use serde_yaml::Value;
use std::collections::BTreeSet;

pub(super) fn available(
    step: &Value,
    checkout: &CheckoutState,
    local_actions: &BTreeSet<String>,
) -> Option<bool> {
    let directory = step
        .get("uses")
        .and_then(Value::as_str)?
        .strip_prefix("./")?;
    Some(checkout.available() && local_actions.contains(directory))
}
