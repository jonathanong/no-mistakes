use super::super::{alias, modules};
use super::support::{catalog, world};
use crate::codebase::dependencies::graph::ImportTarget;

#[test]
fn recorded_import_targets_replace_neighbor_stems() {
    let mut world = world();
    let columns = world.columns.clone();
    let escape = world.escape.clone();
    world
        .graph
        .record_import_target(&escape, "widgets.mts", ImportTarget::File(columns.clone()));
    world
        .graph
        .record_import_target(&escape, "widgets.mts", ImportTarget::File(columns.clone()));
    world
        .graph
        .record_import_target(&escape, "pkg/sql", ImportTarget::Unresolved);
    world.graph.record_import_target(
        &escape,
        "pkg/sql",
        ImportTarget::File(world.widgets.clone()),
    );
    let catalog = catalog(&world);
    assert_eq!(
        modules::module_file(&world.graph, &world.facts, &catalog, &escape, "widgets.mts"),
        Some(columns),
    );
    assert!(
        modules::module_file(&world.graph, &world.facts, &catalog, &escape, "pkg/sql").is_none()
    );
    let open = alias::hop(
        &catalog,
        &world.graph,
        &world.facts,
        &escape,
        "pkg/sql",
        "orderColumns",
        &mut Vec::new(),
    );
    assert_eq!(
        (open.len(), open[0].open_specifier.as_deref()),
        (1, Some("pkg/sql"))
    );
}
