use super::super::coverage::{assemble, blank, call_site, empty_graph};
use super::*;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::{arguments::PathProvenance, TypeScriptModuleFacts};
use crate::fx::FxHashMap;
use std::path::Path;
use std::sync::Arc;

struct Facts(FxHashMap<PathBuf, TsFileFacts>);
impl TsFactLookup for Facts {
    fn get_ts_facts(&self, file: &Path) -> Option<&TsFileFacts> {
        self.0.get(file)
    }
}

fn hint(name: &str) -> EnvDiagnostic {
    EnvDiagnostic {
        name: name.to_string(),
        fallback: Some("./tmp/example".into()),
        captured: false,
        line: 3,
    }
}

#[test]
fn ambiguous_getter_diagnostics_do_not_choose_one_environment_variable() {
    let mut facts = Facts(FxHashMap::default());
    for (file, name) in [("/repo/a.mts", "ROOT_A"), ("/repo/b.mts", "ROOT_B")] {
        let mut origin = PathOrigin::unknown();
        origin.env = Some(name.into());
        origin.env_diagnostic = Some(Box::new(hint(name)));
        let mut paths = PathProvenance::default();
        paths.env_getters.insert(4, origin);
        facts.0.insert(
            file.into(),
            TsFileFacts {
                module_bindings: Some(Arc::new(TypeScriptModuleFacts {
                    path_provenance: Some(Arc::new(paths)),
                    ..TypeScriptModuleFacts::default()
                })),
                ..TsFileFacts::default()
            },
        );
    }
    let mut first = call_site(Path::new("/repo/query.mts"), "query", 1);
    first.target_node = Some(NodeId::callable("/repo/a.mts", "root", CallableId(4)));
    let mut second = first.clone();
    second.target_node = Some(NodeId::callable("/repo/b.mts", "root", CallableId(4)));
    assert_eq!(
        getter_environment_hint(&first, &facts).unwrap().name,
        "ROOT_A"
    );
    assert_eq!(
        getter_environment_hint(&second, &facts).unwrap().name,
        "ROOT_B"
    );
    first.alternatives.push(second);
    assert!(getter_environment_hint(&first, &facts).is_none());
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let mut origin = PathOrigin::unknown();
    origin.getter = Some(first.offset);
    assert!(matches!(
        owners.ownership(&origin, &first, &facts, &[]),
        super::super::Ownership::No
    ));
}

#[test]
fn an_unknown_entry_does_not_name_a_production_location_as_the_test_culprit() {
    let cause = EnvCause {
        hint: hint("ROOT"),
        file: "/repo/production.mts".into(),
        line: 7,
        reason: "clears that environment variable before reaching this scan",
        concrete: false,
    };
    let message = cause.message(Path::new("/repo"), "readdirSync");
    assert!(message.contains("process.env.ROOT"));
    assert!(message.contains("literal fallback \"./tmp/example\""));
    assert!(message.contains("a concrete test or setup entry cannot be identified"));
    assert!(message.contains("make the test-to-scan call or import route explicit"));
    assert!(!message.contains("production.mts:7"));
    assert!(!message.contains("clears that environment variable"));
}

#[test]
fn a_known_static_import_remains_a_concrete_location_without_a_call_offset() {
    let cause = EnvCause {
        hint: hint("ROOT"),
        file: "/repo/test.mts".into(),
        line: 4,
        reason: "has no proven assignment before module initialization",
        concrete: true,
    };
    let message = cause.message(Path::new("/repo"), "readdirSync");
    assert!(message.contains("test.mts:4 has no proven assignment before module initialization"));
    assert!(message.contains("Fix: at test.mts:4"));
}
