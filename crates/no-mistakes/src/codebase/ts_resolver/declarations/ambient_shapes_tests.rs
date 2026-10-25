use super::{AmbientModules, ClassLookup};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::{Path, PathBuf};

fn shapes_path() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/ambient-shapes/shapes.d.ts"),
    )
}

fn lookup(modules: &AmbientModules, specifier: &str, exported: &str) -> ClassLookup {
    modules.class(specifier, Path::new("importer.ts"), exported, None)
}

#[test]
fn nested_export_assignment_requires_exported_ancestors() {
    let path = shapes_path();
    let facts = collect_ts_facts(
        std::slice::from_ref(&path),
        TsFactPlan {
            function_calls: true,
            ..TsFactPlan::default()
        },
    );
    let file = facts
        .get(&path)
        .unwrap_or_else(|| panic!("missing facts for {}", path.display()));
    let mut modules = AmbientModules::default();
    modules.index_file(None, &path, file);
    let classes: Vec<_> = file
        .ancestry
        .classes
        .iter()
        .map(|class| {
            (
                class.namespace.join("."),
                class.name.clone(),
                class.module.clone(),
            )
        })
        .collect();

    assert!(
        matches!(
            lookup(&modules, "spec:nested", "Inner.NestedBase"),
            ClassLookup::Unique(_, _)
        ),
        "{classes:?} assignments={:?}",
        file.ancestry.ambient_module_assignments
    );
    assert!(matches!(
        lookup(&modules, "spec:direct", "Direct"),
        ClassLookup::Unique(_, _)
    ));
    assert!(matches!(
        lookup(&modules, "spec:multi", "Left"),
        ClassLookup::Unique(_, _)
    ));
    // One class is proof for that name only. A second class makes every other name ambiguous.
    assert!(matches!(
        lookup(&modules, "spec:direct", "Other"),
        ClassLookup::Missing
    ));
    assert!(matches!(
        lookup(&modules, "spec:multi", "NoSuch"),
        ClassLookup::Ambiguous
    ));
    assert!(matches!(
        lookup(&modules, "spec:sealed", "Inner.NestedBase"),
        ClassLookup::Missing
    ));
    assert!(matches!(
        lookup(&modules, "spec:absent", "Direct"),
        ClassLookup::Missing
    ));

    modules.index_file(None, &path, file);
    assert!(matches!(
        lookup(&modules, "spec:direct", "Direct"),
        ClassLookup::Unique(_, _)
    ));
    // The same export from another file is not proof, including for a name the module lacks.
    let other = path.with_file_name("other.d.ts");
    modules.index_file(None, &other, file);
    assert!(matches!(
        lookup(&modules, "spec:direct", "Direct"),
        ClassLookup::Ambiguous
    ));
    assert!(matches!(
        lookup(&modules, "spec:direct", "Other"),
        ClassLookup::Ambiguous
    ));
}
