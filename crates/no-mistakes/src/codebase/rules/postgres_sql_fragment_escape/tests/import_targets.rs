use super::super::{alias, modules, resolve};
use super::support::{catalog, ctx, fragment_facts, insert, world};
use crate::codebase::dependencies::graph::{merge_import_target, ImportTarget};

#[test]
fn recorded_import_targets_replace_neighbor_stems() {
    let mut world = world();
    let columns = world.columns.clone();
    let escape = world.escape.clone();
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "widgets.mts",
        ImportTarget::File(columns.clone()),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "widgets.mts",
        ImportTarget::File(columns.clone()),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "pkg/sql",
        ImportTarget::Unresolved,
    );
    merge_import_target(
        &mut world.graph.import_targets,
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

#[test]
fn resolved_tag_import_keeps_a_fragment_exported_under_that_name() {
    let mut world = world();
    let producer = world.columns.clone();
    insert(
        &mut world.facts,
        producer.clone(),
        fragment_facts(&[("default", 10)]),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &producer,
        "@example/sql",
        ImportTarget::File(producer.clone()),
    );
    let catalog = catalog(&world);
    let file_ctx = ctx(&world, &catalog, &producer);
    let found = resolve::name_fragments(&file_ctx, "sql");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].exported, "default");
    assert!(found[0].open_specifier.is_none());

    let mut open_world = super::support::world();
    let producer = open_world.columns.clone();
    insert(
        &mut open_world.facts,
        producer.clone(),
        fragment_facts(&[("default", 10)]),
    );
    let open_catalog = super::support::catalog(&open_world);
    let open_ctx = ctx(&open_world, &open_catalog, &producer);
    assert!(resolve::name_fragments(&open_ctx, "sql").is_empty());
}
