use super::super::cause::CauseKind;
use super::super::consumers::Consumed;
use super::super::coverage::{assemble, blank, call_site, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use super::super::route_index::RouteHit;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::NodeId;
use crate::fx::FxHashMap;
use std::path::Path;

enum Hole {
    Local,
    Reference,
    Routed,
    FileWide,
}

fn caller_hole(hole: Hole) -> super::super::Cause {
    let enumerator = NodeId::callable("/repo/src/fn.mts", "enumerate", CallableId(1));
    let caller = NodeId::callable("/repo/src/caller.mts", "caller", CallableId(2));
    let file = caller.as_file().unwrap();
    let site = call_site(file, "caller", 2);
    let graph = empty_graph();
    let mut callers = FxHashMap::default();
    callers.insert(enumerator.clone(), vec![&site]);
    let mut fixture = blank(&graph);
    match hole {
        Hole::Local => fixture.escaped = Escapes::with_local(file, "caller"),
        Hole::Reference => fixture.escaped = Escapes::with_reference(file, "caller", 9),
        Hole::Routed => {
            fixture.consumed = Consumed::with_hits(vec![RouteHit::nondirect(
                Path::new("/repo/tests/t.mts"),
                "caller",
                file,
            )]);
        }
        Hole::FileWide => fixture.consumed = Consumed::containing(file),
    }
    fixture.callers = callers;
    assemble(fixture)
        .caller_blocked(&enumerator, &NoFacts)
        .expect("caller")
}

#[test]
fn a_non_symbol_parameter_is_not_proven() {
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    assert!(!owners.parameter_proven(&NodeId::module("pkg")));
}

#[test]
fn an_escaped_reference_names_that_line() {
    let cause = caller_hole(Hole::Reference);
    assert_eq!(cause.kind, CauseKind::NonDirectReference);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("src/caller.mts:9 references `caller`"),
        "{text}"
    );
}

#[test]
fn an_escape_without_a_reference_names_the_function() {
    let cause = caller_hole(Hole::Local);
    assert_eq!(cause.kind, CauseKind::NonDirectReference);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("src/caller.mts:1 references `caller`"),
        "{text}"
    );
}

#[test]
fn a_routed_non_call_names_that_import() {
    let cause = caller_hole(Hole::Routed);
    assert_eq!(cause.kind, CauseKind::NonDirectReference);
    let text = cause.detail(Path::new("/repo"));
    assert!(text.contains("tests/t.mts:8 references `caller`"), "{text}");
}

#[test]
fn a_consumed_file_without_a_hit_stays_unproven() {
    let cause = caller_hole(Hole::FileWide);
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("callers are not proven (src/caller.mts:1 declares it)"),
        "{text}"
    );
}
