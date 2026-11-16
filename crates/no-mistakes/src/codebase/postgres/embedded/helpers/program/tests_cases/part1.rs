use super::*;

#[test]
fn strings_and_fresh_builders() {
    let p = load("exec.ts");
    assert_eq!(one(&p, "fresh", &[SiteArg::Scalar]), "sql:SELECT $1");
    assert_eq!(one(&p, "rawSplice", &[lit("p")]), "str:p.x IS NULL");
    fails(&p, "rawSplice", &[SiteArg::Scalar]);
    fails(&p, "rawSplice", &[lit("two\nlines")]);
    fails(&p, "rawSplice", &[SiteArg::Bool(true)]);
    assert_eq!(one(&p, "literalString", &[]), "str:a = 1");
    assert_eq!(
        one(&p, "concatenated", &[lit("a"), lit("b")]),
        "str:a AND b"
    );
    // Nothing in it can contribute SQL, so it never lowers.
    assert!(!p.contains_key("numbers"));
}

#[test]
fn builder_parameters_accumulate_a_suffix() {
    let p = load("exec.ts");
    let b = SiteArg::Builder;
    assert_eq!(
        one(&p, "appender", std::slice::from_ref(&b)),
        "void+0: AND a = 1"
    );
    assert_eq!(
        one(&p, "returnsBuilder", std::slice::from_ref(&b)),
        "param0+0: AND b = 1"
    );
    assert_eq!(
        one(&p, "aliasOfParameter", std::slice::from_ref(&b)),
        "param0+0: AND c = 1"
    );
    assert_eq!(
        one(&p, "inlinesFragment", &[b.clone(), SiteArg::Scalar]),
        "void+0: AND b = $1"
    );
    // A caller-owned builder can be appended to but never read back.
    fails(&p, "readsBuilderParameter", std::slice::from_ref(&b));
    fails(&p, "interpolatesBuilderParameter", std::slice::from_ref(&b));
    // A builder needs a parameter annotated as the SQLStatement type.
    fails(&p, "appendsToNonBuilder", std::slice::from_ref(&b));
    fails(&p, "appendsToNonBuilder", &[lit("x")]);
    fails(&p, "appendsNumber", &[b]);
    fails(&p, "interpolatesFreeName", &[]);
}

#[test]
fn unknown_tests_fork_and_known_ones_decide() {
    let p = load("exec.ts");
    let both = exec(&p, "branches", &[SiteArg::Scalar]).unwrap();
    assert_eq!(
        both,
        ["sql:SELECT 1 WHERE a = 1", "sql:SELECT 1 WHERE b = 1"]
    );
    assert_eq!(
        one(&p, "branches", &[SiteArg::Bool(true)]),
        "sql:SELECT 1 WHERE a = 1"
    );
    assert_eq!(
        one(&p, "branches", &[SiteArg::Bool(false)]),
        "sql:SELECT 1 WHERE b = 1"
    );
    assert_eq!(one(&p, "branches", &[lit("")]), "sql:SELECT 1 WHERE b = 1");
    assert_eq!(one(&p, "branches", &[lit("x")]), "sql:SELECT 1 WHERE a = 1");
    assert_eq!(
        one(&p, "branches", &[SiteArg::Null]),
        "sql:SELECT 1 WHERE b = 1"
    );
    assert_eq!(
        one(&p, "branches", &[object(&[])]),
        "sql:SELECT 1 WHERE a = 1"
    );
    // The same unknown test decides the same way each time it is asked.
    let x = object(&[("x", SiteArg::Scalar)]);
    assert_eq!(
        exec(&p, "repeatedTest", &[x]).unwrap(),
        ["sql:SELECT 1 AND a = 1 AND b = 1", "sql:SELECT 1 AND c = 1"]
    );
}

#[test]
fn comparisons_and_logic_over_known_values() {
    let p = load("exec.ts");
    let known = |x, y| object(&[("x", x), ("y", y)]);
    assert_eq!(
        one(&p, "negatedTests", &[known(SiteArg::Bool(true), lit("z"))]),
        "sql:SELECT 1 AND b = 1 AND c = 1 AND d = 1"
    );
    assert_eq!(
        one(
            &p,
            "negatedTests",
            &[known(SiteArg::Undefined, SiteArg::Null)]
        ),
        "sql:SELECT 1 AND a = 1"
    );
    assert_eq!(
        one(&p, "negatedTests", &[SiteArg::Undefined]),
        "sql:SELECT 1 AND a = 1"
    );
    // Unknown values leave both outcomes of each comparison open.
    assert!(exec(&p, "negatedTests", &[SiteArg::Scalar]).unwrap().len() > 1);
    let t = SiteArg::Bool(true);
    let f = SiteArg::Bool(false);
    assert_eq!(
        one(&p, "logicalTests", &[t.clone(), f.clone()]),
        "sql:SELECT 1 AND either = 1"
    );
    assert_eq!(
        one(&p, "logicalTests", &[f.clone(), f.clone()]),
        "sql:SELECT 1"
    );
    assert_eq!(
        one(&p, "logicalTests", &[t.clone(), t]),
        "sql:SELECT 1 AND both = 1 AND either = 1"
    );
    assert_eq!(
        exec(&p, "logicalTests", &[SiteArg::Scalar, f])
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn logic_as_values_and_statements() {
    let p = load("exec.ts");
    let s = SiteArg::Scalar;
    assert!(exec(&p, "valueLogic", &[s.clone(), s.clone()]).is_some());
    assert!(exec(&p, "valueLogic", &[SiteArg::Null, lit("x")]).is_some());
    assert!(exec(&p, "valueLogic", &[lit("x"), SiteArg::Undefined]).is_some());
    let b = SiteArg::Builder;
    // The same unknown flag decides both statements the same way.
    assert_eq!(
        exec(&p, "statementLogic", &[SiteArg::Scalar, b]).unwrap(),
        ["void+1: AND a = 1 AND b = 1", "void+1: AND c = 1"]
    );
}

#[test]
fn ternary_text_and_aliases() {
    let p = load("exec.ts");
    assert_eq!(
        exec(&p, "ternaryText", &[SiteArg::Scalar, lit("p")]).unwrap(),
        ["sql:SELECT 1 WHERE p.a = 1", "sql:SELECT 1 WHERE p.b = 1"]
    );
    assert_eq!(
        one(&p, "ternaryText", &[SiteArg::Bool(false), lit("p")]),
        "sql:SELECT 1 WHERE p.b = 1"
    );
    fails(&p, "ternaryText", &[SiteArg::Scalar, SiteArg::Scalar]);
}

#[test]
fn throwing_ends_a_path() {
    let p = load("exec.ts");
    assert_eq!(one(&p, "throwsOnUnsafe", &[lit("ok")]), "sql:SELECT 1");
    throws(&p, "throwsOnUnsafe", &[lit("bad")]);
    assert_eq!(
        exec(&p, "throwsOnUnsafe", &[SiteArg::Scalar]).unwrap(),
        ["sql:SELECT 1"]
    );
    throws(&p, "alwaysThrows", &[]);
}

#[test]
fn returns_and_blocks() {
    let p = load("exec.ts");
    assert_eq!(
        exec(&p, "earlyReturn", &[SiteArg::Scalar]).unwrap(),
        ["sql:SELECT 1", "sql:SELECT 1 WHERE a = 1"]
    );
    assert_eq!(
        exec(&p, "nestedBlock", &[SiteArg::Scalar]).unwrap(),
        ["sql:SELECT 1 WHERE a = 1", "sql:SELECT 1 WHERE b = 1"]
    );
}
