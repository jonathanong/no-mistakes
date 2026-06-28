fn print_edge_paths(edges: &[Edge]) {
    let paths: BTreeSet<&str> = edges
        .iter()
        .flat_map(|e| [e.from.as_str(), e.to.as_str()])
        .collect();
    for p in paths {
        println!("{p}");
    }
}
