use super::*;

#[test]
fn defaults_apply_to_missing_and_undefined_arguments() {
    let p = load("exec.ts");
    let strict = "sql:SELECT 1 WHERE strict = 1 /* x! */";
    assert_eq!(one(&p, "withDefault", &[lit("x")]), strict);
    assert_eq!(
        one(&p, "withDefault", &[lit("x"), SiteArg::Undefined]),
        strict
    );
    assert_eq!(
        one(&p, "withDefault", &[lit("x"), SiteArg::Bool(false)]),
        "sql:SELECT 1 /* x! */"
    );
    assert_eq!(
        one(
            &p,
            "withDefault",
            &[lit("x"), SiteArg::Bool(false), lit("y")]
        ),
        "sql:SELECT 1 /* y */"
    );
}

#[test]
fn objects_arrays_and_members() {
    let p = load("exec.ts");
    let a = object(&[("a", lit("v"))]);
    assert_eq!(
        one(&p, "members", std::slice::from_ref(&a)),
        "sql:SELECT 1 /* v */"
    );
    fails(&p, "members", &[]);
    fails(&p, "members", &[SiteArg::Scalar]);
    assert_eq!(
        one(&p, "memberOfLiteral", &[object(&[("table", lit("t"))])]),
        "sql:SELECT 1 FROM t"
    );
    fails(
        &p,
        "memberOfLiteral",
        &[object(&[("table", SiteArg::Scalar)])],
    );
    fails(&p, "memberOfLiteral", &[object(&[])]);
    assert_eq!(
        one(&p, "memberOfUnknown", &[SiteArg::Scalar]),
        "sql:SELECT $1, $2"
    );
    fails(&p, "memberOfString", &[lit("abc")]);
    assert_eq!(
        one(&p, "spreads", std::slice::from_ref(&a)),
        "sql:SELECT 1x"
    );
    assert_eq!(one(&p, "spreadsNothing", &[object(&[])]), "sql:SELECT 1z");
    assert_eq!(
        one(&p, "spreadsNothing", &[SiteArg::Scalar]),
        "sql:SELECT 1z"
    );
    fails(&p, "spreadsString", &[]);
    fails(&p, "computedKey", &[]);
    assert_eq!(
        one(&p, "arrayLength", &[SiteArg::Array(vec![lit("a")])]),
        "sql:SELECT $1"
    );
    assert_eq!(
        one(
            &p,
            "arrayElementsAreBound",
            &[SiteArg::Array(vec![lit("a")])]
        ),
        "sql:SELECT $1"
    );
}

#[test]
fn loops_unroll_over_known_arrays() {
    let p = load("exec.ts");
    let two = SiteArg::Array(vec![lit("a"), lit("b")]);
    assert_eq!(
        one(&p, "sumOver", std::slice::from_ref(&two)),
        "sql:SELECT 1 UNION SELECT 1 FROM a UNION SELECT 1 FROM b"
    );
    assert_eq!(
        one(&p, "sumOver", &[SiteArg::Array(vec![])]),
        "sql:SELECT 1"
    );
    fails(&p, "sumOver", &[SiteArg::Scalar]);
    fails(&p, "sumOver", &[SiteArg::Array(vec![lit("a"); 17])]);
    assert_eq!(one(&p, "returnsInsideLoop", &[two]), "sql:SELECT 1 FROM a");
    assert_eq!(
        one(&p, "returnsInsideLoop", &[SiteArg::Array(vec![])]),
        "sql:SELECT 0"
    );
    fails(&p, "loopOverUnknown", &[SiteArg::Scalar]);
}

#[test]
fn calls_between_helpers() {
    let p = load("exec.ts");
    assert_eq!(
        one(&p, "callsFresh", &[SiteArg::Scalar]),
        "sql:WITH x AS (SELECT $1)"
    );
    assert_eq!(
        one(&p, "callsAppender", &[SiteArg::Builder]),
        "param0+0: AND a = 1"
    );
    assert_eq!(
        one(&p, "callsReturningBuilder", &[SiteArg::Builder]),
        "param0+0: AND b = 1"
    );
    // An unresolved statement call with plain arguments cannot change SQL.
    assert_eq!(
        one(&p, "callsUnresolvedStatement", &[lit("a")]),
        "sql:SELECT 1"
    );
    fails(&p, "callsUnresolvedValue", &[lit("a")]);
    fails(&p, "callsUnresolvedWithObject", &[SiteArg::Builder]);
    assert_eq!(
        one(&p, "passesObjectsAndArrays", &[lit("v")]),
        "sql:SELECT 1v"
    );
    fails(&p, "passesBuilderInObject", &[SiteArg::Builder]);
    fails(&p, "callsThroughParameter", &[SiteArg::Scalar]);
    fails(&p, "callsFailing", &[]);
    fails(&p, "selfRecursive", &[]);
}

#[test]
fn unsupported_statements_fail_only_when_reached() {
    let p = load("exec.ts");
    for name in ["usesAssignment", "opaqueTaken"] {
        fails(&p, name, &[lit("a")]);
    }
    assert_eq!(one(&p, "usesSwitch", &[lit("a")]), "sql:SELECT 1");
    fails(&p, "usesSwitch", &[SiteArg::Scalar]);
    fails(&p, "usesSwitch", &[lit("missing")]);
    // These keep their SQL inside statements the model does not know, so
    // there is nothing left to run.
    for name in ["usesTry", "usesVar"] {
        assert!(!p.contains_key(name), "{name}");
    }
    assert_eq!(one(&p, "untaken", &[SiteArg::Bool(false)]), "sql:SELECT 1");
    fails(&p, "untaken", &[SiteArg::Bool(true)]);
    assert_eq!(one(&p, "usesAwait", &[]), "sql:SELECT 1");
    fails(&p, "usesTemplateTag", &[]);
    fails(&p, "usesRawTag", &[]);
    fails(&p, "usesDestructuring", &[SiteArg::Scalar]);
    assert!(!p.contains_key("emptyReturn"));
    assert_eq!(
        one(&p, "voidBody", &[SiteArg::Builder]),
        "void+0: AND a = 1"
    );
}
