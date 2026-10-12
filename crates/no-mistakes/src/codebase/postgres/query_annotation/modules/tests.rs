use super::Module;
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::sync::Arc;

#[test]
fn indexes_keep_the_first_runtime_binding_and_explicit_export() {
    let binding = |specifier: &str, is_type_only| ImportedBinding {
        specifier: specifier.into(),
        local: "name".into(),
        imported: "value".into(),
        kind: ImportedBindingKind::Named,
        is_type_only,
    };
    let ts = Arc::new(TsFileFacts {
        imported_bindings: vec![
            binding("type", true),
            binding("first", false),
            binding("second", false),
        ],
        exported_bindings: vec![
            ExportedBinding {
                specifier: None,
                local: "first".into(),
                exported: "name".into(),
            },
            ExportedBinding {
                specifier: Some("second".into()),
                local: "second".into(),
                exported: "name".into(),
            },
        ],
        ..Default::default()
    });
    let module = Module::new(Arc::clone(&ts));
    assert!(
        Arc::ptr_eq(&module.ts, &ts),
        "retain the prepared facts without copying"
    );
    assert_eq!(module.imports["name"], 1);
    assert_eq!(module.exports["name"], 0);
    assert!(!module.targets.contains_key("type"));
    assert!(
        module.targets.values().all(|target| target.get().is_none()),
        "construction must not resolve unused syntax"
    );
}
