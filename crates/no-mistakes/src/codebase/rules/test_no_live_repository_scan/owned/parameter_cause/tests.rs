use super::super::cause::CauseKind;
use super::super::coverage::{assemble, blank, call_site, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::NodeId;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{
    ArgumentText, PathAnchor, PathOrigin, PathProvenance,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

fn parameter_origin(owner: u32) -> PathOrigin {
    PathOrigin {
        anchor: PathAnchor::Unknown,
        segments: Vec::new(),
        dynamic: false,
        env: None,
        getter: None,
        root_tails: Vec::new(),
        param: Some(0),
        param_member: None,
        param_owner: Some(owner),
        owner_line: Some(3),
        reassigned_at: None,
        ..PathOrigin::unknown()
    }
}

#[test]
fn an_escaped_caller_blocks_a_proven_parameter() {
    let enumerator = NodeId::callable("/repo/src/fn.mts", "enumerate", CallableId(11));
    let enum_file = enumerator.as_file().unwrap().to_path_buf();
    let caller_file = NodeId::callable("/repo/tests/t.mts", "testFn", CallableId(2))
        .as_file()
        .unwrap()
        .to_path_buf();
    let caller_site = call_site(&caller_file, "testFn", 2);
    let site = call_site(&enum_file, "enumerate", 11);
    let graph = empty_graph();
    let mut callers = FxHashMap::default();
    callers.insert(enumerator.clone(), vec![&caller_site]);
    let mut nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>> = FxHashMap::default();
    nodes.entry(enum_file).or_default().insert(11, enumerator);
    let mut fixture = blank(&graph);
    fixture.escaped = Escapes::with_local(&caller_file, "testFn");
    fixture.callers = callers;
    fixture.nodes = nodes;
    let cause = assemble(fixture)
        .param_cause(0, &parameter_origin(11), &site, &NoFacts, &[])
        .expect_err("caller");
    assert_eq!(cause.kind, CauseKind::NonDirectReference);
    let text = cause.detail(std::path::Path::new("/repo"));
    assert!(text.contains("tests/t.mts:1 references `testFn`"), "{text}");
}

#[test]
fn a_file_wide_consume_names_the_forwarded_argument() {
    use crate::codebase::dependencies::extract::CallableId;
    use crate::codebase::ts_source::facts::TsFileFacts;
    use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
    use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
    use std::sync::Arc;

    let list_file = PathBuf::from("/repo/src/list.mts");
    let caller_file = PathBuf::from("/repo/src/x.mts");
    let list_files = NodeId::callable(&list_file, "listFiles", CallableId(11));
    let gather = NodeId::callable(&caller_file, "gather", CallableId(20));
    let scan = call_site(&list_file, "listFiles", 11);
    let mut forwarded_site = call_site(&caller_file, "gather", 20);
    forwarded_site.line = 94;
    forwarded_site.offset = 50;
    let graph = empty_graph();
    let mut callers = FxHashMap::default();
    callers.insert(list_files.clone(), vec![&forwarded_site]);
    let mut nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>> = FxHashMap::default();
    nodes
        .entry(list_file.clone())
        .or_default()
        .insert(11, list_files);
    nodes
        .entry(caller_file.clone())
        .or_default()
        .insert(20, gather);
    let mut fixture = blank(&graph);
    fixture.consumed = super::super::consumers::Consumed::containing(&list_file);
    fixture.callers = callers;
    fixture.nodes = nodes;
    let mut provenance = PathProvenance::default();
    let mut origin = parameter_origin(20);
    origin.param_owner = Some(20);
    provenance.arg_paths.insert(50, vec![(0, origin)]);
    provenance.arg_text.insert(
        50,
        vec![ArgumentText {
            index: 0,
            property: None,
            line: forwarded_site.line,
            text: "root".to_string(),
        }],
    );
    let facts = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            path_provenance: Some(Arc::new(provenance)),
            ..TypeScriptModuleFacts::default()
        })),
        ..TsFileFacts::default()
    };
    let cause = assemble(fixture)
        .param_cause(0, &parameter_origin(11), &scan, &ArgFacts(facts), &[])
        .expect_err("forwarded");
    assert_eq!(cause.kind, CauseKind::ForwardedParameter);
    let text = cause.detail(std::path::Path::new("/repo"));
    assert!(text.contains("src/x.mts:94"), "{text}");
    assert!(text.contains("`root`"), "{text}");
    assert!(!text.contains("declares it"), "{text}");
}

#[test]
fn an_argument_without_a_parameter_source_is_forwarded() {
    let caller_file = PathBuf::from("/repo/src/x.mts");
    let mut forwarded_site = call_site(&caller_file, "gather", 20);
    forwarded_site.offset = 50;
    let graph = empty_graph();
    let mut provenance = PathProvenance::default();
    provenance
        .arg_paths
        .insert(50, vec![(0, parameter_origin(20))]);
    provenance.arg_text.insert(
        50,
        vec![ArgumentText {
            index: 0,
            property: None,
            line: forwarded_site.line,
            text: "root".to_string(),
        }],
    );
    let facts = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            path_provenance: Some(Arc::new(provenance)),
            ..TypeScriptModuleFacts::default()
        })),
        ..TsFileFacts::default()
    };
    let steps =
        assemble(blank(&graph)).caller_steps(&forwarded_site, 0, None, None, &ArgFacts(facts), &[]);
    let [super::Step::Block(cause)] = steps.as_slice() else {
        panic!("forwarded block");
    };
    assert_eq!(cause.kind, CauseKind::ForwardedParameter);
}

struct ArgFacts(crate::codebase::ts_source::facts::TsFileFacts);

impl crate::codebase::dependencies::graph::TsFactLookup for ArgFacts {
    fn get_ts_facts(
        &self,
        _: &std::path::Path,
    ) -> Option<&crate::codebase::ts_source::facts::TsFileFacts> {
        Some(&self.0)
    }
}
