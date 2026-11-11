use super::super::coverage::{assemble, blank, call_site, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use super::*;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::NodeId;
use std::path::Path;

#[test]
fn unknown_suffixes_require_complete_named_caller_proof() {
    let graph = empty_graph();
    let file = Path::new("/repo/query.mts");
    let site = call_site(file, "query", 1);
    let mut tail = PathOrigin::unknown();
    assert!(!assemble(blank(&graph)).safe_suffix(&tail, &site, &NoFacts, 0));
    tail.param = Some(0);
    tail.param_member = Some("table".into());
    assert!(!assemble(blank(&graph)).safe_suffix(&tail, &site, &NoFacts, 0));
    tail.param_member = None;
    assert!(!assemble(blank(&graph)).safe_suffix(&tail, &site, &NoFacts, 0));
    tail.param_owner = Some(1);
    let source = NodeId::callable(file, "query", CallableId(1));
    let mut fixture = blank(&graph);
    fixture
        .nodes
        .entry(file.to_path_buf())
        .or_default()
        .insert(1, source.clone());
    fixture.escaped = Escapes::with_local(file, "query");
    assert!(!assemble(fixture).safe_suffix(&tail, &site, &NoFacts, 0));
    let caller = call_site(Path::new("/repo/test.mts"), "test", 2);
    let mut fixture = blank(&graph);
    fixture
        .nodes
        .entry(file.to_path_buf())
        .or_default()
        .insert(1, source.clone());
    fixture.callers.insert(source, vec![&caller]);
    assert!(!assemble(fixture).safe_suffix(&tail, &site, &NoFacts, 0));
}

#[test]
fn an_explicit_unknown_argument_does_not_use_the_omitted_argument_default() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFileFacts;
    use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
    use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
    use std::sync::Arc;

    struct Facts(TsFileFacts);
    impl TsFactLookup for Facts {
        fn get_ts_facts(&self, _: &Path) -> Option<&TsFileFacts> {
            Some(&self.0)
        }
    }
    let graph = empty_graph();
    let file = Path::new("/repo/query.mts");
    let site = call_site(file, "query", 1);
    let caller = call_site(Path::new("/repo/test.mts"), "test", 2);
    let source = NodeId::callable(file, "query", CallableId(1));
    let mut fixture = blank(&graph);
    fixture
        .nodes
        .entry(file.to_path_buf())
        .or_default()
        .insert(1, source.clone());
    fixture.callers.insert(source, vec![&caller]);
    let owners = assemble(fixture);
    let mut tail = PathOrigin::unknown();
    tail.param = Some(0);
    tail.param_owner = Some(1);
    let mut default = PathOrigin::unknown();
    default.anchor = PathAnchor::Relative;
    default.dynamic = false;
    default.segments = vec!["table".into()];
    let mut paths = PathProvenance::default();
    paths
        .parameter_defaults
        .entry(1)
        .or_default()
        .insert(0, default);
    let mut facts = Facts(TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            path_provenance: Some(Arc::new(paths.clone())),
            ..TypeScriptModuleFacts::default()
        })),
        ..TsFileFacts::default()
    });
    assert!(owners.safe_suffix(&tail, &site, &facts, 0));
    paths.arg_text.insert(0, vec![(0, "unprojectable".into())]);
    Arc::make_mut(facts.0.module_bindings.as_mut().unwrap()).path_provenance =
        Some(Arc::new(paths));
    assert!(!owners.safe_suffix(&tail, &site, &facts, 0));
}
