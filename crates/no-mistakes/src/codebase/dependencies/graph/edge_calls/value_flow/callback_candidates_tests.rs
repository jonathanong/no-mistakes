use super::*;

fn callable() -> FlowValue {
    FlowValue::Callable {
        file: std::path::PathBuf::from("owner.mts"),
        scope: "owner".into(),
        id: crate::codebase::dependencies::extract::CallableId(1),
    }
}

fn export(path: &str) -> FlowValue {
    FlowValue::Target(ResolvedCallTarget::ModuleExport {
        specifier: "pkg".into(),
        export_path: path.into(),
        repository_target: None,
        callable_id: None,
    })
}

fn alternatives(paths: &[&str], has_unknown: bool) -> FlowValue {
    let known = paths
        .iter()
        .map(|path| ResolvedCallCandidate {
            target: KnownCallTarget::ModuleExport {
                specifier: "pkg".into(),
                export_path: (*path).into(),
                repository_target: None,
                callable_id: None,
            },
            node: None,
        })
        .collect();
    FlowValue::Target(ResolvedCallTarget::Alternatives(std::sync::Arc::new(
        ResolvedCallAlternatives::new(known, has_unknown),
    )))
}

fn same(left: FlowValue, right: FlowValue) {
    assert!(left == right);
}

fn kept_exports(value: FlowValue) -> Vec<String> {
    let FlowValue::Target(ResolvedCallTarget::Alternatives(alternatives)) = value else {
        panic!("expected alternatives");
    };
    assert!(!alternatives.has_unknown);
    alternatives
        .known()
        .iter()
        .map(|candidate| match &candidate.target {
            KnownCallTarget::ModuleExport { export_path, .. } => export_path.clone(),
            _ => panic!("expected module export"),
        })
        .collect()
}

#[test]
fn ownership_candidates_keep_known_callables_and_drop_unknown_values() {
    same(select(vec![]), FlowValue::Unknown);
    same(select(vec![callable()]), callable());
    same(
        select(vec![callable(), FlowValue::Unknown, export("")]),
        callable(),
    );
    assert!(matches!(
        select(vec![callable(), export("run")]),
        FlowValue::Choice(values) if values.len() == 2
    ));
    same(select(vec![export("run")]), export("run"));
    same(select(vec![export("")]), FlowValue::Unknown);
    same(
        select(vec![FlowValue::Target(ResolvedCallTarget::Unknown)]),
        FlowValue::Unknown,
    );
    assert_eq!(
        kept_exports(select(vec![alternatives(&["", "run"], true)])),
        ["run"]
    );
    same(select(vec![alternatives(&[""], false)]), FlowValue::Unknown);
}

#[test]
fn ownership_candidates_unwrap_known_executors_inside_flow_wrappers() {
    let wrapped = [
        FlowValue::Possible(Box::new(callable())),
        FlowValue::Choice(vec![callable(), FlowValue::Unknown]),
        FlowValue::Bound {
            target: Box::new(callable()),
            arguments: vec![],
        },
        FlowValue::Returned {
            factory: vec![],
            executor: Box::new(callable()),
            receiver: None,
        },
        FlowValue::TypedObject {
            executor: Box::new(callable()),
            properties: rustc_hash::FxHashMap::default(),
        },
        FlowValue::Provenance {
            executor: Box::new(callable()),
            typed: vec![],
            members: vec![],
            receiver_factories: vec![],
        },
    ];
    for value in wrapped {
        assert!(select(vec![value]) != FlowValue::Unknown);
    }
    for value in [
        FlowValue::Possible(Box::new(FlowValue::Unknown)),
        FlowValue::Choice(vec![FlowValue::Unknown, export("")]),
        FlowValue::Bound {
            target: Box::new(FlowValue::Unknown),
            arguments: vec![],
        },
        FlowValue::Returned {
            factory: vec![],
            executor: Box::new(FlowValue::Unknown),
            receiver: None,
        },
        FlowValue::TypedObject {
            executor: Box::new(FlowValue::Unknown),
            properties: rustc_hash::FxHashMap::default(),
        },
        FlowValue::Provenance {
            executor: Box::new(FlowValue::Unknown),
            typed: vec![],
            members: vec![],
            receiver_factories: vec![],
        },
        FlowValue::Uncallable,
        FlowValue::Null,
        FlowValue::Promise(Box::new(callable())),
    ] {
        same(select(vec![value]), FlowValue::Unknown);
    }
}
