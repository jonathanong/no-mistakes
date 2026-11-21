use super::super::{alias, modules, resolve, visible};
use super::support::{catalog, ctx, fragment_facts, import, insert, world};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::{ExportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{merge_import_target, ImportTarget};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::sync::Arc;

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
        modules::module_file(
            Some(&world.graph),
            &world.facts,
            &catalog,
            &escape,
            "widgets.mts"
        ),
        Some(columns),
    );
    assert!(modules::module_file(
        Some(&world.graph),
        &world.facts,
        &catalog,
        &escape,
        "pkg/sql"
    )
    .is_none());
    let open = alias::hop(
        &catalog,
        Some(&world.graph),
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
fn a_bare_specifier_without_a_graph_is_not_an_unresolved_warning() {
    let world = world();
    let catalog = catalog(&world);
    let open = alias::hop(
        &catalog,
        None,
        &world.facts,
        &world.escape,
        "pkg/sql",
        "orderColumns",
        &mut Vec::new(),
    );
    assert!(open.is_empty(), "{open:?}");
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

#[test]
fn a_reexported_namespace_without_a_recorded_target_stays_quiet_or_open() {
    let world = world();
    let catalog = catalog(&world);
    let mut ts = TsFileFacts::default();
    ts.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "columns".to_string(),
        exported: "orderColumns".to_string(),
    });
    ts.imported_bindings.push(import(
        "pkg/sql",
        "columns",
        "*",
        ImportedBindingKind::Namespace,
    ));
    let unproven = alias::source_alias(
        &catalog,
        None,
        &world.facts,
        &world.escape,
        &ts,
        "orderColumns",
        &mut Vec::new(),
    );
    assert!(unproven.is_empty(), "{unproven:?}");

    ts.imported_bindings[0].specifier = "./missing".to_string();
    let open = alias::source_alias(
        &catalog,
        Some(&world.graph),
        &world.facts,
        &world.escape,
        &ts,
        "orderColumns",
        &mut Vec::new(),
    );
    assert_eq!(
        (open.len(), open[0].open_specifier.as_deref()),
        (1, Some("./missing"))
    );
}

#[test]
fn a_bare_star_reexport_without_a_graph_is_not_an_unresolved_warning() {
    let mut world = world();
    {
        let entry = world.facts.ts.get_mut(&world.index).expect("index facts");
        let mut owned = (*entry.ts).clone();
        owned.star_reexport_specifiers = vec!["pkg/sql".to_string()];
        *entry = Arc::new(CheckFileFacts {
            ts: Arc::new(owned),
            ..CheckFileFacts::default()
        });
    }
    let catalog = catalog(&world);
    let starred = alias::star_fragments(
        &catalog,
        None,
        &world.facts,
        &world.index,
        &[],
        &mut Vec::new(),
    );
    assert!(starred
        .iter()
        .all(|fragment| fragment.open_specifier.is_none()));
    let all = visible::visible_all(&catalog, None, &world.facts, &world.index, &mut Vec::new());
    assert!(all
        .iter()
        .all(|fragment| fragment.open_specifier.as_deref() != Some("pkg/sql")));
    let one = visible::visible(
        &catalog,
        None,
        &world.facts,
        &world.index,
        "notExported",
        &mut Vec::new(),
    );
    assert!(one.iter().all(|fragment| fragment.open_specifier.is_none()));
}

#[test]
fn declare_module_skips_an_empty_name_and_an_unclosed_quote() {
    let specs = modules::declare_module_specifiers(
        "declare module \"\" {}\ndeclare module 'kept' {}\ndeclare module \"open",
    );
    assert_eq!(specs, vec!["kept".to_string()]);
}
