use super::*;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, NodeId};
use crate::codebase::ts_resolver::TsConfig;

#[test]
fn layered_typed_aliases_retain_concrete_package_classes_and_exact_error_identity() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("shared-package-targets/fixture");
    let graph = DepGraph::build_with_plan(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            paths: vec![
                ("@fixture/*".into(), vec!["src/*".into()]),
                ("@fixture-core/*".into(), vec!["src/core/*".into()]),
            ],
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            extends: true,
            ..Default::default()
        },
    )
    .unwrap();
    for (callee, declaration) in [
        (
            "Configured",
            "node_modules/utility-constructors/config.d.ts",
        ),
        ("Filter", "node_modules/utility-constructors/filter.d.ts"),
    ] {
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| {
                site.file.ends_with("src/typed-alias-use.ts") && site.source_callee == callee
            })
            .unwrap();
        let target = graph
            .constructor_class(&site.file, site.offset)
            .expect("a symbolic module target alone is insufficient proof");
        let NodeId::Symbol {
            file,
            symbol,
            callable_id: Some(id),
        } = target
        else {
            panic!("{target:?}");
        };
        assert!(file.ends_with(declaration), "{target:?}");
        assert_eq!(symbol.as_ref(), callee);
        assert!(graph.class_is_root(file, symbol, Some(*id)), "{target:?}");
    }
    let class = |name| {
        graph
            .class_declarations()
            .iter()
            .find(|class| class.file.ends_with("src/errors.ts") && class.scope == name)
            .unwrap()
            .node()
    };
    for (callee, used, unused) in [
        ("ConstructedError", "UsedError", "UnusedError"),
        ("ConstructedTwin", "UsedTwin", "UnusedTwin"),
    ] {
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| site.file.ends_with("src/error-use.ts") && site.source_callee == callee)
            .unwrap();
        let target = graph.constructor_class(&site.file, site.offset).unwrap();
        assert_eq!(target, &class(used));
        assert_ne!(target, &class(unused));
    }
}
