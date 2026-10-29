use super::support::fixture_with_effects;
use std::{path::PathBuf, sync::Arc};

#[test]
fn duplicate_prepared_import_identity_rejects_returned_parameter_projection() {
    let (source, mut ts, heads) = fixture_with_effects("returned-param-ambiguity.mts");
    assert_eq!(heads.pending_returned_params.len(), 1);
    let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
    let import = module
        .imports
        .iter_mut()
        .find(|item| item.specifier == "./other.mjs")
        .unwrap();
    import.bindings.push(import.bindings[0].clone());
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads/returned-param-ambiguity.mts");
    let calls = heads
        .calls
        .keys()
        .copied()
        .filter(|span| source[span.0 as usize..].starts_with("read("))
        .collect::<Vec<_>>();
    let changed = crate::ast::with_program(&path, &source, |program, _| {
        super::collect_with_helpers(
            program,
            &ts,
            calls.into_iter(),
            &Default::default(),
            true,
            true,
        )
    })
    .unwrap();
    assert!(changed.pending_returned_params.is_empty());
}

#[test]
fn returned_parameter_roots_require_continuation_safe_heads() {
    use super::super::HeadProof;
    let (source, _, heads) = fixture_with_effects("returned-param-continuation.mts");
    assert_eq!(super::support::assert_markers(&source, &heads), 6);
    // Only stable roots enter the prepared source-hop queue. EOF BEGIN and
    // partial/empty comments must remain unproven regardless of helper identity.
    let proofs = heads
        .pending_returned_params
        .iter()
        .map(|p| p.proof)
        .collect::<Vec<_>>();
    assert_eq!(
        proofs,
        vec![HeadProof::Missing, HeadProof::Literal, HeadProof::Missing]
    );
}

#[test]
fn returned_parameter_grammar_rejects_identity_and_continuation_boundaries() {
    use oxc_ast::ast::Statement;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads/returned-param-grammar.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut checked = 0;
        for statement in &program.body {
            let Statement::FunctionDeclaration(function) = statement else {
                continue;
            };
            let name = function.id.as_ref().unwrap().name.as_str();
            assert_eq!(
                super::super::returned_params::function(function).is_some(),
                name == "accepted",
                "{name}"
            );
            checked += 1;
        }
        assert_eq!(checked, 25);
    })
    .unwrap();
}

#[test]
fn returned_parameter_call_roots_enforce_budgets_and_plain_native_syntax() {
    use oxc_ast::ast::Statement;
    use oxc_span::GetSpan;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads/returned-param-call-grammar.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut checked = 0;
        for statement in &program.body {
            let Statement::VariableDeclaration(declaration) = statement else {
                continue;
            };
            let binding = &declaration.declarations[0];
            let name = &source[binding.id.span().start as usize..binding.id.span().end as usize];
            assert_eq!(
                super::super::returned_params::root(binding.init.as_ref().unwrap()).is_some(),
                name.starts_with("accepted"),
                "{name}"
            );
            checked += 1;
        }
        assert_eq!(checked, 7);
    })
    .unwrap();
}

#[test]
fn returned_parameter_exports_reject_untrusted_defaults_and_dynamic_owners() {
    let (_, _, heads) = fixture_with_effects("returned-param-export-controls.mts");
    assert_eq!(
        heads
            .exported_returned_params
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["accepted"]
    );
}

#[test]
fn ambiguous_or_missing_export_metadata_never_proves_returned_identity() {
    let name = "returned-param-export-controls.mts";
    let (source, ts, _) = fixture_with_effects(name);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads")
        .join(name);
    for missing_module in [false, true] {
        let mut ts = ts.clone();
        if missing_module {
            ts.module_bindings = None;
        } else {
            let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
            let export = module
                .exports
                .iter()
                .find(|item| item.exported == "accepted")
                .unwrap()
                .clone();
            module.exports.push(export);
        }
        let heads = crate::ast::with_program(&path, &source, |program, _| {
            super::collect_with_helpers(
                program,
                &ts,
                std::iter::empty(),
                &Default::default(),
                true,
                true,
            )
        })
        .unwrap();
        assert!(heads.exported_returned_params.is_empty());
    }
}
