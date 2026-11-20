use super::super::namespace_uses::value_sites;
use super::support::{assert_from, catalog, ctx, import, world};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::ImportedBindingKind;
use std::sync::Arc;

fn offsets(world: &mut super::support::World, sites: Vec<(&str, u32)>) {
    let entry = world.facts.ts.get_mut(&world.escape).expect("escape facts");
    let mut owned = (*entry.ts).clone();
    owned.namespaces.value_use_offsets = sites
        .into_iter()
        .map(|(name, offset)| (name.to_string(), offset))
        .collect();
    *entry = Arc::new(CheckFileFacts {
        ts: Arc::new(owned),
        ..CheckFileFacts::default()
    });
}

#[test]
fn a_namespace_value_use_reports_every_fragment() {
    let mut world = world();
    offsets(&mut world, vec![("columns", 40)]);
    let catalog = catalog(&world);
    let ctx = ctx(&world, &catalog, &world.escape);
    let sites = value_sites(&ctx);
    assert_eq!(sites.len(), 1);
    assert_eq!(sites[0].0, 40);
    assert_from(
        &sites[0].1,
        &world.columns,
        &["orderColumns", "widgetColumns"],
    );
}

#[test]
fn a_named_import_value_use_stays_with_the_call_scan() {
    let mut world = world();
    {
        let entry = world.facts.ts.get_mut(&world.escape).expect("escape facts");
        let mut owned = (*entry.ts).clone();
        owned.imported_bindings.push(import(
            "./columns",
            "widgetColumns",
            "widgetColumns",
            ImportedBindingKind::Named,
        ));
        owned
            .namespaces
            .value_use_offsets
            .push(("widgetColumns".to_string(), 8));
        *entry = Arc::new(CheckFileFacts {
            ts: Arc::new(owned),
            ..CheckFileFacts::default()
        });
    }
    let catalog = catalog(&world);
    let ctx = ctx(&world, &catalog, &world.escape);
    assert!(value_sites(&ctx).is_empty());
}

#[test]
fn an_unresolved_namespace_value_use_stays_open() {
    let mut world = world();
    {
        let entry = world.facts.ts.get_mut(&world.escape).expect("escape facts");
        let mut owned = (*entry.ts).clone();
        owned.imported_bindings.push(import(
            "./missing",
            "gone",
            "*",
            ImportedBindingKind::Namespace,
        ));
        owned
            .namespaces
            .value_use_offsets
            .push(("gone".to_string(), 12));
        *entry = Arc::new(CheckFileFacts {
            ts: Arc::new(owned),
            ..CheckFileFacts::default()
        });
    }
    let catalog = catalog(&world);
    let ctx = ctx(&world, &catalog, &world.escape);
    let sites = value_sites(&ctx);
    assert_eq!(sites.len(), 1);
    assert_eq!(sites[0].1[0].open_specifier.as_deref(), Some("./missing"));
}
