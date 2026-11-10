use super::{function_fallback, CauseKind};
use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::{
    NodeId, ResolvedCallSite, ResolvedCallTarget, TsFactLookup,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{PathOrigin, PathProvenance};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use std::path::Path;
use std::sync::Arc;

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
    let cause = function_fallback(&callers, &source, &NoFacts, None);
    assert_eq!(cause.kind, CauseKind::RecursiveHelper);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("src/fn.mts:1 calls itself through a recursive inner helper"),
        "{text}"
    );
    assert!(cause.fix().contains("recursive helper"));
}

#[test]
fn callee_that_calls_itself_does_not_name_the_outer_function() {
    let outer = node("listFiles", 2);
    let inner = node("visit", 3);
    let from_outer = site("listFiles", 2);
    let from_inner = site("visit", 3);
    let mut callers = FxHashMap::default();
    callers.insert(inner, vec![&from_outer, &from_inner]);
    let cause = function_fallback(&callers, &outer, &NoFacts, Some(0));
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
}

#[test]
fn a_callee_that_does_not_call_itself_stays_unproven() {
    let outer = node("snapshotFiles", 2);
    let inner = node("visit", 3);
    let from_outer = site("snapshotFiles", 2);
    let mut callers = FxHashMap::default();
    callers.insert(inner, vec![&from_outer]);
    let cause = function_fallback(&callers, &outer, &NoFacts, None);
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("callers are not proven (src/fn.mts:1 declares it)"),
        "{text}"
    );
    assert!(cause.fix().contains("owned root"));
}

struct ArgFacts(TsFileFacts);

impl TsFactLookup for ArgFacts {
    fn get_ts_facts(&self, _: &Path) -> Option<&TsFileFacts> {
        Some(&self.0)
    }
}

fn origin(param: Option<usize>, owner: u32) -> PathOrigin {
    PathOrigin {
        param,
        param_owner: Some(owner),
        ..PathOrigin::unknown()
    }
}

fn facts(proven: bool, param: Option<usize>) -> ArgFacts {
    let mut provenance = PathProvenance::default();
    let entry = vec![(0, origin(param, 2))];
    if proven {
        provenance.arg_paths.insert(0, entry);
    } else {
        provenance.arg_unproven.insert(0, entry);
    }
    let mut module = TypeScriptModuleFacts::default();
    module.path_provenance = Some(Arc::new(provenance));
    ArgFacts(TsFileFacts {
        module_bindings: Some(Arc::new(module)),
        ..TsFileFacts::default()
    })
}

fn recursive_pair() -> (NodeId, NodeId, ResolvedCallSite, ResolvedCallSite) {
    let outer = node("listFiles", 2);
    let inner = node("visit", 3);
    let from_outer = site("listFiles", 2);
    let from_inner = site("visit", 3);
    (outer, inner, from_outer, from_inner)
}

#[test]
fn a_parameter_passed_into_a_recursive_callee_is_the_helper() {
    let (outer, inner, from_outer, from_inner) = recursive_pair();
    let mut callers = FxHashMap::default();
    callers.insert(inner, vec![&from_outer, &from_inner]);
    let facts = facts(true, Some(0));
    let cause = function_fallback(&callers, &outer, &facts, Some(0));
    assert_eq!(cause.kind, CauseKind::RecursiveHelper);
}

#[test]
fn a_captured_parameter_beside_recursion_is_not_the_helper() {
    let (outer, inner, from_outer, from_inner) = recursive_pair();
    let mut callers = FxHashMap::default();
    callers.insert(inner, vec![&from_outer, &from_inner]);
    let facts = facts(false, None);
    let cause = function_fallback(&callers, &outer, &facts, Some(0));
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
}

#[test]
fn a_self_call_that_forwards_the_parameter_stays_recursive() {
    let source = node("visit", 2);
    let own = site("visit", 2);
    let mut callers = FxHashMap::default();
    callers.insert(source.clone(), vec![&own]);
    let facts = facts(true, Some(0));
    let cause = function_fallback(&callers, &source, &facts, Some(0));
    assert_eq!(cause.kind, CauseKind::RecursiveHelper);
}

#[test]
fn a_self_call_of_a_different_parameter_is_not_this_root() {
    let source = node("visit", 2);
    let own = site("visit", 2);
    let mut callers = FxHashMap::default();
    callers.insert(source.clone(), vec![&own]);
    let facts = facts(false, Some(1));
    let cause = function_fallback(&callers, &source, &facts, Some(0));
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
}
