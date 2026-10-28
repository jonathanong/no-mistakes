use super::support::{assert_markers, fixture, recollect};
use std::sync::Arc;

const FIXTURE: &str = "helper-ambiguity.mts";

#[test]
fn duplicate_export_names_and_local_owners_do_not_publish_helper_proofs() {
    let (source, original, heads) = fixture(FIXTURE);
    assert_eq!(assert_markers(&source, &heads), 2);
    assert_eq!(heads.exported_helpers.len(), 2);
    assert_eq!(heads.pending_imports.len(), 1);

    let mut duplicate_export = original.clone();
    let module = Arc::make_mut(duplicate_export.module_bindings.as_mut().unwrap());
    let alias = module
        .exports
        .iter()
        .find(|export| export.exported == "alias")
        .unwrap()
        .clone();
    module.exports.push(alias);
    let heads = recollect(FIXTURE, &duplicate_export);
    assert!(!heads.exported_helpers.contains_key("alias"));
    assert!(heads.exported_helpers.contains_key("stable"));

    let mut duplicate_owner = original;
    let module = Arc::make_mut(duplicate_owner.module_bindings.as_mut().unwrap());
    let mut owner = module
        .bindings
        .iter()
        .find(|binding| binding.name == "stable" && !binding.imported)
        .unwrap()
        .clone();
    owner.id = module.bindings.len();
    module.bindings.push(owner);
    let heads = recollect(FIXTURE, &duplicate_owner);
    assert!(!heads.exported_helpers.contains_key("stable"));
    assert!(!heads.exported_helpers.contains_key("alias"));
}

#[test]
fn duplicate_import_binding_does_not_project_an_arbitrary_helper() {
    let (_, mut ts, heads) = fixture(FIXTURE);
    assert_eq!(heads.pending_imports.len(), 1);
    let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
    let import = module
        .imports
        .iter_mut()
        .find(|import| import.specifier == "./external.mjs")
        .unwrap();
    import.bindings.push(import.bindings[0].clone());
    let heads = recollect(FIXTURE, &ts);
    assert!(heads.pending_imports.is_empty());
    assert!(heads.exported_helpers.contains_key("stable"));
}
