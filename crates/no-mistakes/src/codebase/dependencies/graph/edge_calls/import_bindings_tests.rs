use super::*;
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::ts_source::facts::TsFileFacts;

fn binding(
    local: &str,
    kind: ImportedBindingKind,
    is_type_only: bool,
    ordinal: usize,
) -> ImportedBinding {
    ImportedBinding {
        local: local.to_string(),
        specifier: format!("./module-{ordinal}"),
        imported: match kind {
            ImportedBindingKind::Named => format!("Export{ordinal}"),
            ImportedBindingKind::Default => "default".to_string(),
            ImportedBindingKind::Namespace => "*".to_string(),
        },
        kind,
        is_type_only,
    }
}

#[test]
fn import_projections_borrow_canonical_fact_string_buffers() {
    let facts = TsFileFacts {
        imported_bindings: [
            ("named", ImportedBindingKind::Named, false),
            ("default", ImportedBindingKind::Default, false),
            ("namespace", ImportedBindingKind::Namespace, false),
            ("typeNamed", ImportedBindingKind::Named, true),
            ("typeDefault", ImportedBindingKind::Default, true),
            ("typeNamespace", ImportedBindingKind::Namespace, true),
        ]
        .into_iter()
        .enumerate()
        .map(|(ordinal, (local, kind, type_only))| binding(local, kind, type_only, ordinal))
        .collect(),
        ..Default::default()
    };
    let index = CallableFileIndex::from_facts(&facts);
    for source in &facts.imported_bindings {
        for (map, runtime) in [(&index.imported, true), (&index.type_imported, false)] {
            if runtime && source.is_type_only {
                continue;
            }
            let (key, projected) = map.get_key_value(source.local.as_str()).unwrap();
            assert_eq!(
                key.as_ptr(),
                source.local.as_ptr(),
                "the map key must borrow the fact string"
            );
            assert_eq!(projected.specifier.as_ptr(), source.specifier.as_ptr());
            assert_eq!(projected.local.as_ptr(), source.local.as_ptr());
            assert_eq!(projected.imported.as_ptr(), source.imported.as_ptr());
            assert_eq!(projected.specifier, source.specifier);
        }
    }
    assert!(!index.type_imported.contains_key("missing"));
}

fn binding_view(binding: &ImportedBinding) -> (&str, &str, &str, ImportedBindingKind, bool) {
    (
        &binding.specifier,
        &binding.local,
        &binding.imported,
        binding.kind,
        binding.is_type_only,
    )
}

#[test]
fn type_projection_preserves_independent_last_runtime_and_last_any_winners() {
    for kind in [
        ImportedBindingKind::Named,
        ImportedBindingKind::Default,
        ImportedBindingKind::Namespace,
    ] {
        // Recovered syntax may repeat a local. Preserve both historical winners,
        // including the runtime binding hidden by a later type-only declaration.
        for sequence in [
            &[][..],
            &[false],
            &[true],
            &[false, true],
            &[true, false],
            &[false, true, false],
            &[true, false, true],
            &[false, false],
            &[true, true],
        ] {
            let mut bindings = vec![binding("other", ImportedBindingKind::Namespace, true, 99)];
            bindings.extend(
                sequence
                    .iter()
                    .enumerate()
                    .map(|(ordinal, is_type_only)| binding("local", kind, *is_type_only, ordinal)),
            );
            let facts = TsFileFacts {
                imported_bindings: bindings,
                ..Default::default()
            };
            // These are the original two independent map projections, not the
            // borrowed projection under test.
            let runtime: FxHashMap<_, _> = facts
                .imported_bindings
                .iter()
                .filter(|binding| !binding.is_type_only)
                .map(|binding| (binding.local.clone(), binding.clone()))
                .collect();
            let any: FxHashMap<_, _> = facts
                .imported_bindings
                .iter()
                .map(|binding| (binding.local.clone(), binding.clone()))
                .collect();
            let index = CallableFileIndex::from_facts(&facts);
            assert_eq!(index.imported.len(), runtime.len());
            assert_eq!(index.type_imported.len(), any.len());
            for local in ["local", "other", "missing"] {
                assert_eq!(
                    index
                        .imported
                        .get(local)
                        .map(|binding| binding_view(binding)),
                    runtime.get(local).map(binding_view),
                    "{kind:?} {sequence:?}"
                );
            }
            for local in ["local", "other", "missing"] {
                assert_eq!(
                    index
                        .type_imported
                        .get(local)
                        .map(|binding| binding_view(binding)),
                    any.get(local).map(binding_view),
                    "{kind:?} {sequence:?} {local}"
                );
                assert_eq!(
                    index.type_imported.contains_key(local),
                    any.contains_key(local)
                );
            }
        }
    }
}
