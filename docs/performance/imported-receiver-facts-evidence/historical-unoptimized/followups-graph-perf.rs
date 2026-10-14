use no_mistakes::codebase::dependencies::{graph::{DepGraph, GraphBuildPlan, ResolvedCallTarget}, TsConfig};
use std::{hint::black_box, path::PathBuf, hash::{Hash, Hasher}};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let root = PathBuf::from(&args[1]);
    let iterations: usize = args[2].parse().unwrap();
    let config = TsConfig { dir: root.clone(), paths: vec![], paths_dir: root.clone(), base_url: None };
    let mut signature = String::new();
    for _ in 0..iterations {
        let graph = DepGraph::build_with_plan(&root, &config, GraphBuildPlan { calls: true, ..Default::default() }).unwrap();
        let sites = graph.resolved_call_sites();
        let mut digest = std::collections::hash_map::DefaultHasher::new();
        format!("{sites:?}").hash(&mut digest);
        signature = format!("{}:{}:{:016x}", sites.len(), sites.iter().filter(|site| matches!(site.target, ResolvedCallTarget::Unknown)).count(), digest.finish());
        black_box(graph);
    }
    println!("{signature}");
}
