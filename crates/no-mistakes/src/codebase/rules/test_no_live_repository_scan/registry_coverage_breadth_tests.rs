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
