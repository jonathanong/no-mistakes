use super::super::super::local_actions::{LocalActionCatalog, LocalActionKind};
use super::checkout::CheckoutState;
use serde_yaml::Value;

pub(super) fn available(
    step: &Value,
    checkout: &CheckoutState,
    local_actions: &LocalActionCatalog,
    runner_os: Option<&str>,
) -> Option<bool> {
    let directory = step
        .get("uses")
        .and_then(Value::as_str)?
        .strip_prefix("./")?;
    let kind = local_actions.kind(directory);
    Some(
        checkout.available()
            && kind.is_some()
            && (kind != Some(LocalActionKind::Docker) || runner_os == Some("Linux")),
    )
}
