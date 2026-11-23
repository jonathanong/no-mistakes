use super::super::{alias, modules, resolve, visible};
use super::support::{assert_from, catalog, ctx, fragment_facts, import, insert, world};
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
    assert!(open.is_empty());
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
fn a_reexported_namespace_without_a_resolved_target_has_no_proven_fragments() {
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
    assert!(open.is_empty());
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
    assert_from(&starred, &world.columns, &["orderColumns", "widgetColumns"]);
    let all = visible::visible_all(&catalog, None, &world.facts, &world.index, &mut Vec::new());
    assert_from(&all, &world.columns, &["orderColumns", "widgetColumns"]);
    let one = visible::visible(
        &catalog,
        None,
        &world.facts,
        &world.index,
        "notExported",
        &mut Vec::new(),
    );
    assert!(one.is_empty());
}

#[test]
fn declare_module_skips_an_empty_name_and_an_unclosed_quote() {
    let specs = modules::declare_module_specifiers(
        "declare module \"\" {}\ndeclare module 'kept' {}\ndeclare module \"open",
    );
    assert_eq!(specs, vec!["kept".to_string()]);
}

#[test]
fn external_targets_are_silent_and_project_gaps_are_recorded() {
    let mut world = world();
    let escape = world.escape.clone();
    for specifier in ["node:fs", "orders-widget"] {
        merge_import_target(
            &mut world.graph.import_targets,
            &escape,
            specifier,
            ImportTarget::External,
        );
    }
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "./missing-widget",
        ImportTarget::Unresolved,
    );
    let mut catalog = catalog(&world);
    for specifier in ["node:fs", "orders-widget"] {
        let found = alias::hop(
            &catalog,
            Some(&world.graph),
            &world.facts,
            &escape,
            specifier,
            "widgetColumns",
            &mut Vec::new(),
        );
        assert!(found.is_empty(), "{specifier}");
    }
    assert!(catalog.take_unresolved().is_empty());
    let open = alias::hop(
        &catalog,
        Some(&world.graph),
        &world.facts,
        &escape,
        "./missing-widget",
        "widgetColumns",
        &mut Vec::new(),
    );
    assert!(open.is_empty());
    let gaps = catalog.take_unresolved();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps.get(&escape).unwrap().value(), "./missing-widget");
}

#[test]
fn one_ambient_module_names_an_external_specifier() {
    let mut world = world();
    let escape = world.escape.clone();
    let declared = std::path::PathBuf::from("/repo/declared.ts");
    insert(
        &mut world.facts,
        declared.clone(),
        ambient_module("ambient-cols"),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "ambient-cols",
        ImportTarget::External,
    );
    let catalog = catalog(&world);
    assert_eq!(
        modules::module_file(
            Some(&world.graph),
            &world.facts,
            &catalog,
            &escape,
            "ambient-cols"
        ),
        Some(declared)
    );
}

#[test]
fn several_ambient_modules_record_the_ambiguous_specifier() {
    let mut world = world();
    let escape = world.escape.clone();
    insert(
        &mut world.facts,
        std::path::PathBuf::from("/repo/declared-a.ts"),
        ambient_module("orders-widget"),
    );
    insert(
        &mut world.facts,
        std::path::PathBuf::from("/repo/declared-b.ts"),
        ambient_module("orders-widget"),
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "orders-widget",
        ImportTarget::External,
    );
    let mut catalog = catalog(&world);
    let open = alias::hop(
        &catalog,
        Some(&world.graph),
        &world.facts,
        &escape,
        "orders-widget",
        "widgetColumns",
        &mut Vec::new(),
    );
    assert!(open.is_empty());
    let gaps = catalog.take_unresolved();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps.get(&escape).unwrap().value(), "orders-widget");
}

#[test]
fn external_star_reexports_add_no_fragment() {
    let mut world = world();
    let barrel = std::path::PathBuf::from("/repo/widgets/barrel.ts");
    insert(
        &mut world.facts,
        barrel.clone(),
        TsFileFacts {
            star_reexport_specifiers: vec!["node:fs".to_string()],
            ..TsFileFacts::default()
        },
    );
    merge_import_target(
        &mut world.graph.import_targets,
        &barrel,
        "node:fs",
        ImportTarget::External,
    );
    let catalog = catalog(&world);
    assert!(visible::visible_all(
        &catalog,
        Some(&world.graph),
        &world.facts,
        &barrel,
        &mut Vec::new()
    )
    .is_empty());
    assert!(alias::star_fragments(
        &catalog,
        Some(&world.graph),
        &world.facts,
        &barrel,
        &[],
        &mut Vec::new(),
    )
    .is_empty());
    assert!(visible::visible(
        &catalog,
        Some(&world.graph),
        &world.facts,
        &barrel,
        "widgetItems",
        &mut Vec::new(),
    )
    .is_empty());
}

fn ambient_module(specifier: &str) -> TsFileFacts {
    ambient_source(&format!("declare module \"{specifier}\" {{}}\n"))
}

fn ambient_source(source: &str) -> TsFileFacts {
    TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        ..TsFileFacts::default()
    }
}

#[test]
fn spaced_ambient_declaration_names_the_external_specifier() {
    let mut world = world();
    let escape = world.escape.clone();
    let declared = std::path::PathBuf::from("/repo/declared.ts");
    let single = std::path::PathBuf::from("/repo/single.ts");
    insert(
        &mut world.facts,
        declared.clone(),
        ambient_source(
            "declare  module \"orders-widget\" {}\nnotdeclare module \"glued-widget\" {}\n",
        ),
    );
    insert(
        &mut world.facts,
        single.clone(),
        ambient_source("declare\nmodule\n'single-widget' {}\n"),
    );
    for (path, source) in [
        ("/repo/tight.ts", "declaremodule \"tight-widget\" {}\n"),
        (
            "/repo/extra.ts",
            "declare moduleExtra \"extra-widget\" {}\n",
        ),
        ("/repo/nospace.ts", "declare module\"nospace-widget\" {}\n"),
        ("/repo/fn.ts", "declare function \"fn-widget\" {}\n"),
    ] {
        insert(
            &mut world.facts,
            std::path::PathBuf::from(path),
            ambient_source(source),
        );
    }
    for specifier in [
        "orders-widget",
        "glued-widget",
        "single-widget",
        "tight-widget",
        "extra-widget",
        "nospace-widget",
        "fn-widget",
    ] {
        merge_import_target(
            &mut world.graph.import_targets,
            &escape,
            specifier,
            ImportTarget::External,
        );
    }
    let catalog = catalog(&world);
    let hit = |specifier: &str| {
        modules::module_file(
            Some(&world.graph),
            &world.facts,
            &catalog,
            &escape,
            specifier,
        )
    };
    assert_eq!(hit("orders-widget"), Some(declared));
    assert_eq!(hit("single-widget"), Some(single));
    assert_eq!(hit("glued-widget"), None);
    assert_eq!(hit("tight-widget"), None);
    assert_eq!(hit("extra-widget"), None);
    assert_eq!(hit("nospace-widget"), None);
    assert_eq!(hit("fn-widget"), None);
}
