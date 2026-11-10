use super::super::cause::CauseKind;
use super::super::consumers::Consumed;
use super::super::coverage::{assemble, blank, call_site, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use super::super::setups::ConfiguredSetups;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use std::path::Path;
use std::sync::Arc;

#[test]
fn a_gap_without_a_config_file_is_unreadable() {
    let graph = empty_graph();
    let mut fixture = blank(&graph);
    fixture.setups = ConfiguredSetups::unknown_without_gap(Path::new("/repo"));
    let cause = assemble(fixture).route_cause(&NodeId::module("pkg"), &NoFacts);
    assert_eq!(cause.kind, CauseKind::UnreadableConfig);
    let text = cause.detail(Path::new("/repo"));
    assert!(text.contains("cannot be read or does not match"), "{text}");
}

#[test]
fn a_non_file_source_is_not_a_named_function() {
    let graph = empty_graph();
    let cause = assemble(blank(&graph)).route_cause(&NodeId::module("pkg"), &NoFacts);
    assert_eq!(cause.kind, CauseKind::NotNamedFunction);
    let text = cause.detail(Path::new("/repo"));
    assert!(text.contains("is not a named function"), "{text}");
}

#[test]
fn an_escape_without_a_reference_names_the_function() {
    let source = NodeId::callable("/repo/src/fn.mts", "fn", CallableId(1));
    let file = source.as_file().unwrap().to_path_buf();
    let graph = empty_graph();
    let mut fixture = blank(&graph);
    fixture.escaped = Escapes::with_local(&file, "fn");
    let cause = assemble(fixture).route_cause(&source, &NoFacts);
    assert_eq!(cause.kind, CauseKind::NonDirectReference);
    let text = cause.detail(Path::new("/repo"));
    assert!(text.contains("src/fn.mts:1 references `fn`"), "{text}");
}

#[test]
fn a_consumed_file_without_a_hit_falls_back_to_callers() {
    let source = NodeId::callable("/repo/src/fn.mts", "fn", CallableId(1));
    let file = source.as_file().unwrap().to_path_buf();
    let graph = empty_graph();
    let mut fixture = blank(&graph);
    fixture.consumed = Consumed::containing(&file);
    let owners = assemble(fixture);
    assert!(super::super::symbol_block::blocks(
        &owners.consumed,
        &file,
        "fn"
    ));
    assert!(super::super::symbol_block::route(&owners.consumed, &file, "fn").is_none());
    let cause = owners.route_cause(&source, &NoFacts);
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
}

#[test]
fn a_call_with_no_argument_maps_does_not_carry_the_parameter() {
    let source = NodeId::callable("/repo/src/fn.mts", "fn", CallableId(1));
    let site = call_site(source.as_file().unwrap(), "fn", 1);
    let mut callers = FxHashMap::default();
    callers.insert(source.clone(), vec![&site]);
    let facts = StoredFacts(provenance_facts(PathProvenance::default()));
    assert_eq!(
        super::flow::parameter_flows(&callers, &source, &source, &facts, 0),
        None
    );
}

#[test]
fn a_module_source_has_no_callable_span() {
    let source = NodeId::module("pkg");
    assert_eq!(
        super::flow::parameter_flows(&FxHashMap::default(), &source, &source, &NoFacts, 0),
        None
    );
}

fn provenance_facts(provenance: PathProvenance) -> TsFileFacts {
    TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            path_provenance: Some(Arc::new(provenance)),
            ..TypeScriptModuleFacts::default()
        })),
        ..TsFileFacts::default()
    }
}

struct StoredFacts(TsFileFacts);

impl TsFactLookup for StoredFacts {
    fn get_ts_facts(&self, _: &Path) -> Option<&TsFileFacts> {
        Some(&self.0)
    }
}
