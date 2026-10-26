/// Canonical membership occurrences include one executable bridge: a class's
/// constructor. Call collection separately verifies that the owner is a class.
pub(crate) fn is_constructor_bridge(invocation: InvocationKind, callee: &str) -> bool {
    invocation == InvocationKind::Membership && callee == "constructor"
}

fn is_traversable_call(
    index: &CallableFileIndex,
    call: &crate::codebase::dependencies::extract::FunctionCall,
) -> bool {
    (call.invocation != crate::codebase::dependencies::extract::InvocationKind::Membership
        || (is_constructor_bridge(call.invocation, &call.callee)
            && call
                .caller
                .as_ref()
                .is_some_and(|caller| index.class_scopes.contains(caller))))
        && !(call.is_callback
            && call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Construct)
}
