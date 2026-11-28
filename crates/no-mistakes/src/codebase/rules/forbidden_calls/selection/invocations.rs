use super::super::config::Invocation;
use crate::codebase::dependencies::extract::InvocationKind;

pub(super) fn allowed_invocations(configured: &[Invocation]) -> Vec<InvocationKind> {
    let values = configured
        .iter()
        .map(|kind| match kind {
            Invocation::Call => InvocationKind::Call,
            Invocation::Construct => InvocationKind::Construct,
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        vec![InvocationKind::Call]
    } else {
        values
    }
}
