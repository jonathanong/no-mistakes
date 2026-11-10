use super::{function_fallback, CauseKind};
use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::{
    NodeId, ResolvedCallSite, ResolvedCallTarget, TsFactLookup,
};
use crate::fx::FxHashMap;
use std::path::Path;

struct NoFacts;

impl TsFactLookup for NoFacts {
    fn get_ts_facts(&self, _: &Path) -> Option<&crate::codebase::ts_source::facts::TsFileFacts> {
        None
    }
}

fn node(name: &str, id: u32) -> NodeId {
    NodeId::callable("src/fn.mts", name, CallableId(id))
}

fn site(name: &str, id: u32) -> ResolvedCallSite {
    ResolvedCallSite {
        file: "src/fn.mts".into(),
        caller: Some(name.into()),
        caller_id: Some(CallableId(id)),
        line: 4,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: name.into(),
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

#[test]
fn self_call_names_the_recursive_helper() {
    let source = node("visit", 1);
    let own = site("visit", 1);
    let mut callers = FxHashMap::default();
    callers.insert(source.clone(), vec![&own]);
    let cause = function_fallback(&callers, &source, &NoFacts);
    assert_eq!(cause.kind, CauseKind::RecursiveHelper);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("src/fn.mts:1 calls itself through a recursive inner helper"),
        "{text}"
    );
    assert!(cause.fix().contains("recursive helper"));
}

#[test]
fn callee_that_calls_itself_names_the_outer_function() {
    let outer = node("snapshotFiles", 2);
    let inner = node("visit", 3);
    let from_outer = site("snapshotFiles", 2);
    let from_inner = site("visit", 3);
    let mut callers = FxHashMap::default();
    callers.insert(inner, vec![&from_outer, &from_inner]);
    let cause = function_fallback(&callers, &outer, &NoFacts);
    assert_eq!(cause.kind, CauseKind::RecursiveHelper);
}

#[test]
fn a_callee_that_does_not_call_itself_stays_unproven() {
    let outer = node("snapshotFiles", 2);
    let inner = node("visit", 3);
    let from_outer = site("snapshotFiles", 2);
    let mut callers = FxHashMap::default();
    callers.insert(inner, vec![&from_outer]);
    let cause = function_fallback(&callers, &outer, &NoFacts);
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("callers are not proven (src/fn.mts:1 declares it)"),
        "{text}"
    );
    assert!(cause.fix().contains("owned root"));
}
