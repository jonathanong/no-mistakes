use super::*;
use crate::codebase::dependencies::extract::InvocationKind;
use std::path::PathBuf;

fn site(target: ResolvedCallTarget, alternatives: Vec<ResolvedCallSite>) -> ResolvedCallSite {
    ResolvedCallSite {
        file: PathBuf::from("/repo/src/caller.ts"),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "invoke".into(),
        target,
        alternatives,
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

fn global(name: &str) -> ResolvedCallSite {
    site(ResolvedCallTarget::Global { name: name.into() }, Vec::new())
}

#[test]
fn global_callback_is_found_directly_and_through_alternatives() {
    assert!(any_global_callback(&global("Array.from")));
    assert!(!any_global_callback(&global("parseInt")));
    assert!(!any_global_callback(&site(ResolvedCallTarget::Unknown, Vec::new())));
    let ambiguous = site(
        ResolvedCallTarget::Unknown,
        vec![global("parseInt"), global("Array.from")],
    );
    assert!(any_global_callback(&ambiguous));
    let plain = site(ResolvedCallTarget::Unknown, vec![global("parseInt")]);
    assert!(!any_global_callback(&plain));
}
