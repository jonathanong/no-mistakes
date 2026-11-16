use super::{graph, home, locate, resolution, Facts};
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

#[test]
fn function_export_walk_revisits_a_file_from_a_shorter_path() {
    let root = PathBuf::from("/repo/src/root.mts");
    let steps: Vec<PathBuf> = (1..=6)
        .map(|index| PathBuf::from(format!("/repo/src/a{index}.mts")))
        .collect();
    let bridge = PathBuf::from("/repo/src/x.mts");
    let scan = PathBuf::from("/repo/src/y.mts");
    let mut links = vec![
        (root.clone(), steps[0].clone()),
        (root.clone(), bridge.clone()),
    ];
    for pair in steps.windows(2) {
        links.push((pair[0].clone(), pair[1].clone()));
    }
    links.push((steps[5].clone(), bridge.clone()));
    links.push((bridge.clone(), scan.clone()));
    let edges: Vec<(&Path, &Path)> = links
        .iter()
        .map(|(from, to)| (from.as_path(), to.as_path()))
        .collect();
    let mut files = HashMap::new();
    let mut link = |file: &Path, next: &str| {
        files.insert(
            file.to_path_buf(),
            TsFileFacts {
                star_reexport_specifiers: vec![next.to_string()],
                ..TsFileFacts::default()
            },
        );
    };
    link(&root, "./a1.mts");
    for (index, step) in steps.iter().enumerate() {
        let next = index + 2;
        if next <= 6 {
            link(step, &format!("./a{next}.mts"));
        }
    }
    link(&steps[5], "./x.mts");
    files.insert(
        root.clone(),
        TsFileFacts {
            star_reexport_specifiers: vec!["./a1.mts".to_string(), "./x.mts".to_string()],
            ..TsFileFacts::default()
        },
    );
    let mut bridge_facts = TsFileFacts {
        star_reexport_specifiers: vec!["./y.mts".to_string()],
        ..TsFileFacts::default()
    };
    bridge_facts.exported_functions.push("bridge".to_string());
    bridge_facts
        .callable_scope_ids
        .push((CallableId(1), "bridge".to_string()));
    files.insert(bridge.clone(), bridge_facts);
    let mut scan_facts = TsFileFacts::default();
    scan_facts.exported_functions.push("sync".to_string());
    scan_facts
        .callable_scope_ids
        .push((CallableId(2), "sync".to_string()));
    files.insert(scan.clone(), scan_facts);
    let resolution = resolution();
    let visible = PathSet::default();
    let linked = graph(&edges);
    let linked_facts = Facts { files };
    let located = locate(&linked, &linked_facts, &resolution, &visible);
    assert_eq!(
        home::function_exports(&located, &root),
        vec![(bridge, "bridge".to_string()), (scan, "sync".to_string())]
    );
}
