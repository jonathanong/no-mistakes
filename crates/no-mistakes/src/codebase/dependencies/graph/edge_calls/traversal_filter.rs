fn is_traversable_call(
    index: &CallableFileIndex,
    call: &crate::codebase::dependencies::extract::FunctionCall,
) -> bool {
    (call.invocation != crate::codebase::dependencies::extract::InvocationKind::Membership
        || (call.callee == "constructor"
            && call
                .caller
                .as_ref()
                .is_some_and(|caller| index.class_scopes.contains(caller))))
        && !(call.is_callback
            && call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Construct)
}
