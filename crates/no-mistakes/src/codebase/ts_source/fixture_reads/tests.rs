use super::*;

fn literal(id: &str) -> String {
    format!("loadFixture(\"{id}\")")
}

#[test]
fn traces_static_reads_and_keeps_ambiguous_uses() {
    let root = crate::test_support::fixture_root("fixture-reads");
    let path = root.join("reads.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let precise = [
        ("direct", "/size"),
        ("string", "/a~1b~0c"),
        ("number", "/0"),
        ("wrapped", "/size"),
        ("asserted", "/size"),
        ("type-asserted", "/size"),
        ("satisfies", "/size"),
        ("alias", "/details/size"),
        ("destructure", "/a~1b~0c"),
        ("destructure", "/0"),
        ("default", "/details"),
        ("unused-field", "/size"),
        ("repeated", "/size"),
        ("repeated", "/label"),
    ];
    let opaque = [
        "large-number",
        "dynamic",
        "prototype",
        "optional-call",
        "two-args",
        "wrapped-callee",
        "computed-binding",
        "rest",
        "array-pattern",
        "empty-object-pattern",
        "mutable",
        "write-alias",
        "write-field",
        "delete-field",
        "increment",
        "method",
        "construct",
        "spread",
        "iterate",
        "enumerate",
        "whole",
        "alternate",
        "source-only",
        "exported",
        "export-default",
        "awaited",
        "absent",
    ];
    let member = "loader.fetch(\"member\")";
    let optional_callee = "loader?.fetch(\"optional-callee\")";
    let literals = precise
        .iter()
        .map(|(id, _)| literal(id))
        .chain(opaque.map(literal))
        .chain([
            literal("unused-root"),
            member.into(),
            optional_callee.into(),
        ])
        .collect();
    let reads = FixtureReads::collect(&path, &source, &literals);

    let unrelated = ["/unrelated".into()];
    for (id, changed) in precise {
        assert!(
            reads.affects(&source, &literal(id), &[changed.into()]),
            "{id}"
        );
        assert!(!reads.affects(&source, &literal(id), &unrelated), "{id}");
    }
    for id in opaque {
        assert!(reads.affects(&source, &literal(id), &unrelated), "{id}");
    }
    assert!(reads.affects(&source, member, &["/size".into()]));
    assert!(!reads.affects(&source, member, &unrelated));
    assert!(reads.affects(&source, optional_callee, &unrelated));
    assert!(!reads.affects(&source, &literal("unused-root"), &unrelated));
}

#[test]
fn keeps_matches_with_broken_syntax_eval_or_unresolved_loader_arguments() {
    let root = crate::test_support::fixture_root("fixture-reads");
    for id in ["broken", "evaluated", "mixed"] {
        let path = root.join(format!("{id}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let literal = literal(id);
        let reads = FixtureReads::collect(&path, &source, &BTreeSet::from([literal.clone()]));
        assert!(reads.affects(&source, &literal, &["/size".into()]), "{id}");
    }
}
