use super::*;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use std::path::PathBuf;

#[test]
fn prepared_callable_wrappers_preserve_existing_reachability() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source/control-flow-returned-entrypoint-wrappers.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        )
    })
    .unwrap();
    // These internal wrappers keep their established traversal contract. Source
    // callable identities come from the same stored fixture as the public tests.
    let opaque = Expr::OpaqueCallback(Box::new(Expr::Name("wrapped".into())));
    assert!(value_reaches(
        &facts.globals,
        &opaque,
        "runArray",
        &mut BTreeSet::new(),
        &[]
    ));
    let computed = Expr::ComputedMember(
        Box::new(Expr::Name("missing".into())),
        Box::new(Expr::Name("runArray".into())),
    );
    assert!(value_reaches(
        &facts.globals,
        &computed,
        "runArray",
        &mut BTreeSet::new(),
        &[]
    ));
    assert!(!value_reaches(
        &facts.globals,
        &computed,
        "missingTarget",
        &mut BTreeSet::new(),
        &[]
    ));
    let append = Expr::Append(
        Box::new(Expr::Name("missing".into())),
        Box::new(Expr::Name("runArray".into())),
    );
    assert!(value_reaches(
        &facts.globals,
        &append,
        "runArray",
        &mut BTreeSet::new(),
        &[]
    ));
}

#[test]
fn prepared_branch_shadows_do_not_escape_global_callback() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/postgres-facts/executed-source/control-flow-returned-entrypoint-static-branches.ts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        )
    })
    .unwrap();
    for name in [
        "shadowedArm",
        "hoistedShadow",
        "parameterShadow",
        "selfShadow",
        "closureShadow",
    ] {
        assert!(
            !exported_reaches(&facts.globals, name, "contextual", &mut BTreeSet::new()),
            "{name}: {:?}",
            facts.globals[name]
        );
    }
    assert!(
        matches!(facts.globals["contextual"], Expr::Function(_)),
        "{:?}",
        facts.globals["contextual"]
    );
    let Expr::Function(hoisted) = &facts.globals["hoistedShadow"] else {
        panic!("stored hoisted shadow must retain its prepared function");
    };
    assert!(shadowed(hoisted, "contextual"));
}
