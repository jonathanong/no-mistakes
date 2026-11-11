use super::super::coverage::{assemble, blank, call_site, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use super::*;
use crate::codebase::dependencies::extract::CallableId;
use std::path::Path;

#[test]
fn incomplete_getter_metadata_never_establishes_ownership() {
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let site = call_site(Path::new("/repo/query.mts"), "query", 1);
    let mut origin = PathOrigin::unknown();
    assert!(!owners.getter_owned(&origin, &site, &NoFacts));
    origin.getter = Some(12);
    assert!(!owners.getter_owned(&origin, &site, &NoFacts));
    assert!(!owners.getter_call_owned(&site, &site, &NoFacts));
    let mut call = site.clone();
    call.target_node = Some(NodeId::callable("/repo/data.mts", "data", CallableId(4)));
    assert!(!owners.getter_call_owned(&call, &site, &NoFacts));
    let mut fixture = blank(&graph);
    fixture.escaped = Escapes::with_local(Path::new("/repo/data.mts"), "data");
    assert!(!assemble(fixture).getter_call_owned(&call, &site, &NoFacts));
}
