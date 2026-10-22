fn is_traversable_call(
    index: &CallableFileIndex,
    call: &crate::codebase::dependencies::extract::FunctionCall,
) -> bool {
    traversable_with_class_scope(call, |caller| index.class_scopes.contains(caller))
}

pub(crate) fn traversable_source_call(
    file: &crate::codebase::ts_source::facts::TsFileFacts,
    call: &crate::codebase::dependencies::extract::FunctionCall,
) -> bool {
    traversable_with_class_scope(call, |caller| file.class_scopes.iter().any(|scope| scope == caller))
}

fn traversable_with_class_scope(
    call: &crate::codebase::dependencies::extract::FunctionCall,
    is_class_scope: impl Fn(&str) -> bool,
) -> bool {
    (call.invocation != crate::codebase::dependencies::extract::InvocationKind::Membership
        || (call.callee == "constructor"
            && call
                .caller
                .as_ref()
                .is_some_and(|caller| is_class_scope(caller))))
        && !(call.is_callback
            && call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Construct)
}
