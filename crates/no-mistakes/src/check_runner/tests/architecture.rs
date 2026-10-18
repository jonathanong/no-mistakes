fn check_task_sources() -> String {
    // Architecture assertions cover the complete production task module,
    // including helpers split out to keep each Rust source under the size gate.
    [
        include_str!("../../check_tasks.rs"),
        include_str!("../../check_tasks/filesystem.rs"),
        include_str!("../../check_tasks/react.rs"),
    ]
    .concat()
}

#[path = "architecture_ci.rs"]
mod ci;
#[path = "architecture_domains.rs"]
mod domains;
#[path = "architecture_graph.rs"]
mod graph;
#[path = "architecture_preparation.rs"]
mod preparation;
