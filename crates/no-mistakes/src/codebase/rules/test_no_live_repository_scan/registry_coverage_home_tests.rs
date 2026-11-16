use super::{export, graph, home, locate, resolution, site, Facts};
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
fn missing_caller_facts_are_not_a_registry() {
    let graph = graph(&[]);
    let facts = Facts {
        files: HashMap::new(),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    let file = Path::new("/repo/src/missing.mts");
    assert!(home::homes(&locate, &site(file), "workers").is_empty());
    assert!(home::function_exports(&locate, file).is_empty());
}

#[test]
fn binding_search_stops_at_the_hop_cap_and_on_a_cycle() {
    let edges = graph(&[]);
    let facts = Facts {
        files: HashMap::new(),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let located = locate(&edges, &facts, &resolution, &visible);
    let file = PathBuf::from("/repo/src/a.mts");
    let mut seen = vec![(file.clone(), "workers".to_string()); 8];
    assert!(home::binding_local(&located, &file, "workers", &mut seen).is_none());

    let left = PathBuf::from("/repo/src/a.mts");
    let right = PathBuf::from("/repo/src/b.mts");
    let edges = graph(&[(&left, &right), (&right, &left)]);
    let mut files = HashMap::new();
    files.insert(
        left.clone(),
        TsFileFacts {
            exported_bindings: vec![export("workers", "workers", Some("./b.mts"))],
            ..TsFileFacts::default()
        },
    );
    files.insert(
        right.clone(),
        TsFileFacts {
            exported_bindings: vec![export("workers", "workers", Some("./a.mts"))],
            ..TsFileFacts::default()
        },
    );
    let facts = Facts { files };
    let located = locate(&edges, &facts, &resolution, &visible);
    assert!(home::binding_local(&located, &left, "workers", &mut Vec::new()).is_none());
}

#[test]
fn a_namespace_reexport_is_not_an_object_registry() {
    let file = PathBuf::from("/repo/src/barrel.mts");
    let graph = graph(&[]);
    let facts = Facts {
        files: HashMap::from([(
            file.clone(),
            TsFileFacts {
                exported_bindings: vec![export("*", "workers", Some("./scan.mts"))],
                ..TsFileFacts::default()
            },
        )]),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    assert!(home::binding_local(&locate, &file, "workers", &mut Vec::new()).is_none());
}

#[test]
fn star_search_skips_a_missing_specifier_and_a_module_that_lacks_the_name() {
    let barrel = PathBuf::from("/repo/src/barrel.mts");
    let barren = PathBuf::from("/repo/src/barren.mts");
    let scan = PathBuf::from("/repo/src/scan.mts");
    let graph = graph(&[(&barrel, &barren), (&barrel, &scan)]);
    let facts = Facts {
        files: HashMap::from([
            (
                barrel.clone(),
                TsFileFacts {
                    star_reexport_specifiers: vec![
                        "./missing.mts".to_string(),
                        "./barren.mts".to_string(),
                        "./scan.mts".to_string(),
                    ],
                    ..TsFileFacts::default()
                },
            ),
            (barren.clone(), TsFileFacts::default()),
            (
                scan.clone(),
                TsFileFacts {
                    exported_bindings: vec![export("workers", "workers", None)],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    assert_eq!(
        home::binding_local(&locate, &barrel, "workers", &mut Vec::new()),
        Some((scan, "workers".to_string()))
    );
}

#[test]
fn function_export_walk_skips_non_callables_and_stops_on_a_star_cycle() {
    let scan = PathBuf::from("/repo/src/scan.mts");
    let looped = PathBuf::from("/repo/src/loop.mts");
    let graph = graph(&[(&scan, &looped), (&looped, &scan)]);
    let facts = Facts {
        files: HashMap::from([
            (
                scan.clone(),
                TsFileFacts {
                    exported_functions: vec!["marker".to_string(), "sync".to_string()],
                    callable_scope_ids: vec![(CallableId(7), "sync".to_string())],
                    star_reexport_specifiers: vec!["./loop.mts".to_string()],
                    ..TsFileFacts::default()
                },
            ),
            (
                looped.clone(),
                TsFileFacts {
                    star_reexport_specifiers: vec!["./scan.mts".to_string()],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&graph, &facts, &resolution, &visible);
    assert_eq!(
        home::function_exports(&locate, &scan),
        vec![(scan, "sync".to_string())]
    );
}

#[test]
fn a_sourced_named_reexport_follows_the_callable_and_skips_a_namespace() {
    let barrel = PathBuf::from("/repo/src/barrel.mts");
    let scan = PathBuf::from("/repo/src/scan.mts");
    let edges = graph(&[(&barrel, &scan)]);
    let facts = Facts {
        files: HashMap::from([
            (
                barrel.clone(),
                TsFileFacts {
                    exported_bindings: vec![
                        export("sync", "sync", Some("./scan.mts")),
                        export("*", "workers", Some("./scan.mts")),
                        export("gone", "gone", Some("./missing.mts")),
                    ],
                    ..TsFileFacts::default()
                },
            ),
            (
                scan.clone(),
                TsFileFacts {
                    exported_bindings: vec![export("sync", "sync", None)],
                    exported_functions: vec!["sync".to_string()],
                    callable_scope_ids: vec![(CallableId(7), "sync".to_string())],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let resolution = resolution();
    let visible = PathSet::default();
    let locate = locate(&edges, &facts, &resolution, &visible);
    assert_eq!(
        home::function_exports(&locate, &barrel),
        vec![(scan, "sync".to_string())]
    );
}

#[test]
fn function_export_walk_keeps_a_wide_barrel_and_stops_at_chain_depth() {
    let barrel = PathBuf::from("/repo/src/barrel.mts");
    let parts: Vec<PathBuf> = (1..=9)
        .map(|index| PathBuf::from(format!("/repo/src/part-{index}.mts")))
        .collect();
    let edges: Vec<(&Path, &Path)> = parts
        .iter()
        .map(|part| (barrel.as_path(), part.as_path()))
        .collect();
    let mut files = HashMap::new();
    let mut stars = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let number = index + 1;
        stars.push(format!("./part-{number}.mts"));
        let mut facts = TsFileFacts::default();
        if number == 9 {
            facts.exported_functions.push("sync".to_string());
            facts
                .callable_scope_ids
                .push((CallableId(9), "sync".to_string()));
        }
        files.insert(part.clone(), facts);
    }
    files.insert(
        barrel.clone(),
        TsFileFacts {
            star_reexport_specifiers: stars,
            ..TsFileFacts::default()
        },
    );
    let resolution = resolution();
    let visible = PathSet::default();
    let wide = graph(&edges);
    let wide_facts = Facts { files };
    let wide_locate = locate(&wide, &wide_facts, &resolution, &visible);
    assert_eq!(
        home::function_exports(&wide_locate, &barrel),
        vec![(parts[8].clone(), "sync".to_string())]
    );

    let chain: Vec<PathBuf> = (0..9)
        .map(|index| PathBuf::from(format!("/repo/src/chain-{index}.mts")))
        .collect();
    let edges: Vec<(&Path, &Path)> = chain
        .windows(2)
        .map(|pair| (pair[0].as_path(), pair[1].as_path()))
        .collect();
    let mut files = HashMap::new();
    for (index, file) in chain.iter().enumerate() {
        let mut facts = TsFileFacts::default();
        if index + 1 < chain.len() {
            let next = index + 1;
            facts
                .star_reexport_specifiers
                .push(format!("./chain-{next}.mts"));
        }
        if index == 7 {
            facts.exported_functions.push("near".to_string());
            facts
                .callable_scope_ids
                .push((CallableId(7), "near".to_string()));
        }
        if index == 8 {
            facts.exported_functions.push("far".to_string());
            facts
                .callable_scope_ids
                .push((CallableId(8), "far".to_string()));
        }
        files.insert(file.clone(), facts);
    }
    let deep = graph(&edges);
    let deep_facts = Facts { files };
    let deep_locate = locate(&deep, &deep_facts, &resolution, &visible);
    assert_eq!(
        home::function_exports(&deep_locate, &chain[0]),
        vec![(chain[7].clone(), "near".to_string())]
    );
}
