use super::support::fixture_with_effects;

#[test]
fn returned_literal_parameter_exports_require_native_tags_and_stable_identity() {
    let (_, _, heads) = fixture_with_effects("returned-literal-source.mts");
    assert_eq!(
        heads
            .exported_returned_params
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["acceptedEight", "acceptedStrings", "acceptedTags"]
    );
}

#[test]
fn returned_literal_parameter_grammar_is_bounded_and_literal_only() {
    use oxc_ast::ast::{Declaration, Statement};
    let name = "returned-literal-source.mts";
    let (source, _, _) = fixture_with_effects(name);
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads")
        .join(name);
    crate::ast::with_program(&path, &source, |program, _| {
        let mut checked = 0;
        for statement in &program.body {
            let Statement::ExportDeclaration(export) = statement else {
                continue;
            };
            let Declaration::FunctionDeclaration(function) = &export.declaration else {
                continue;
            };
            let name = function.id.as_ref().unwrap().name.as_str();
            let expected = name.starts_with("accepted")
                || matches!(name, "customTag" | "shadowedTag" | "changed");
            assert_eq!(
                super::super::returned_params::function(function).is_some(),
                expected,
                "{name}"
            );
            checked += 1;
        }
        assert_eq!(checked, 26);
    })
    .unwrap();
}

#[test]
fn returned_literal_callers_require_private_lifetimes_and_every_imported_arm() {
    let (source, _, heads) = fixture_with_effects("returned-literal-caller.mts");
    let pending = &heads.pending_returned_params;
    assert_eq!(pending.len(), 4);
    let expected = [
        (
            "distinct",
            vec![("./first.mjs", "first"), ("./second.mjs", "second")],
        ),
        (
            "nested",
            vec![
                ("./first.mjs", "first"),
                ("./second.mjs", "second"),
                ("./first.mjs", "first"),
            ],
        ),
        (
            "unresolved",
            vec![("./first.mjs", "first"), ("./unresolved.mjs", "absent")],
        ),
    ];
    for (name, imports) in expected {
        let prefix = format!("read({name}.text,");
        let proof = pending
            .iter()
            .find(|proof| source[proof.span.0 as usize..].starts_with(&prefix))
            .unwrap();
        assert_eq!(
            proof
                .imports
                .iter()
                .map(|(specifier, imported)| (specifier.as_str(), imported.as_str()))
                .collect::<Vec<_>>(),
            imports,
            "{name}"
        );
    }
    let budget = pending
        .iter()
        .find(|proof| source[proof.span.0 as usize..].starts_with("read(depth8.text,"))
        .unwrap();
    assert_eq!(budget.imports.len(), 9);
    for name in [
        "missingValues",
        "singleField",
        "appendAfter",
        "selfReference",
        "depth9",
    ] {
        let prefix = format!("read({name}.text");
        assert!(
            pending
                .iter()
                .all(|proof| !source[proof.span.0 as usize..].starts_with(&prefix)),
            "{name}"
        );
    }
}

#[test]
fn returned_literal_callers_abstain_at_dynamic_scope_boundaries() {
    let (_, _, heads) = fixture_with_effects("returned-literal-caller-eval.mts");
    assert!(heads.pending_returned_params.is_empty());
}
