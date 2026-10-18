//! Expression values, forks and failures.

use super::{exec, fails, lit, load, object, one};
use crate::codebase::postgres::embedded::helpers::SiteArg;

#[test]
fn values_forks_and_failures_in_expressions() {
    let p = load("exec.ts");
    let s = SiteArg::Scalar;
    assert_eq!(
        one(&p, "numberBinds", std::slice::from_ref(&s)),
        "sql:SELECT $1, $2"
    );
    fails(&p, "addsBuilder", &[]);
    fails(&p, "addsFree", &[]);
    assert_eq!(one(&p, "addsUnknown", &[lit("x")]), "sql:SELECT $1");
    assert_eq!(
        one(&p, "addsUnknown", std::slice::from_ref(&s)),
        "sql:SELECT $1"
    );
    fails(&p, "arrayMember", &[SiteArg::Array(vec![])]);
    // Five independent unknown tests are 32 paths, past the cap of 16.
    fails(&p, "thirtyTwoPaths", std::slice::from_ref(&s));
    let letters = SiteArg::Array(vec![lit("a"), lit("b")]);
    assert_eq!(
        exec(&p, "loopThatForks", &[letters, s.clone()])
            .unwrap()
            .len(),
        3
    );
    let many = SiteArg::Array(vec![lit("a"), lit("b"), lit("c"), lit("d"), lit("e")]);
    fails(&p, "loopThatForks", &[many, s.clone()]);
    // A default that itself forks cannot be bound to one value.
    fails(&p, "forkingDefault", std::slice::from_ref(&s));
    assert_eq!(
        one(&p, "forkingDefault", &[s.clone(), lit("q")]),
        "sql:SELECT 1q"
    );
    assert_eq!(one(&p, "nullish", &[SiteArg::Null]), "sql:SELECT 1x");
    assert_eq!(one(&p, "nullish", &[SiteArg::Undefined]), "sql:SELECT 1x");
    assert_eq!(one(&p, "nullish", &[lit("y")]), "sql:SELECT 1y");
    fails(&p, "nullish", std::slice::from_ref(&s));
    assert_eq!(
        one(&p, "negationValue", &[SiteArg::Bool(true)]),
        "sql:SELECT $1"
    );
    assert_eq!(
        one(&p, "negationValue", std::slice::from_ref(&s)),
        "sql:SELECT $1"
    );
    assert_eq!(
        one(&p, "equalityValue", &[lit("a"), lit("a")]),
        "sql:SELECT $1"
    );
    assert_eq!(one(&p, "equalityValue", &[lit("a"), s]), "sql:SELECT $1");
}

#[test]
fn argument_shapes_a_callee_never_receives() {
    let p = load("exec.ts");
    fails(&p, "callsWithFree", &[]);
    // An object with an unknown spread reaches the callee as an unknown value.
    fails(&p, "passesOpenObject", &[SiteArg::Scalar]);
    fails(&p, "passesNestedBuilder", &[SiteArg::Builder]);
}

#[test]
fn an_unmodeled_expression_fails_wherever_it_appears() {
    let p = load("fail.ts");
    assert!(p.len() >= 30, "{}", p.len());
    // An argument supplied for the defaulted parameter would skip its default.
    fails(&p, "inDefault", &[]);
    // An unknown scalar leaves every branch and short-circuit open.
    for name in p.keys().filter(|name| *name != "inDefault") {
        fails(&p, name, &[SiteArg::Scalar, SiteArg::Scalar]);
    }
}

#[test]
fn equality_decides_only_between_known_primitives() {
    let p = load("exec.ts");
    let outcome = |name: &str, a: SiteArg, b: SiteArg| exec(&p, name, &[a, b]).unwrap();
    let yes = vec!["sql:yes".to_string()];
    let no = vec!["sql:no".to_string()];
    let both = vec!["sql:yes".to_string(), "sql:no".to_string()];
    let (n, u, s) = (SiteArg::Null, SiteArg::Undefined, SiteArg::Scalar);
    assert_eq!(outcome("looseEquality", n.clone(), u.clone()), yes);
    assert_eq!(outcome("looseEquality", lit("a"), n.clone()), no);
    assert_eq!(outcome("looseEquality", s.clone(), n.clone()), both);
    assert_eq!(outcome("looseEquality", lit("a"), lit("a")), yes);
    assert_eq!(
        outcome("looseEquality", SiteArg::Bool(true), SiteArg::Bool(true)),
        yes
    );
    assert_eq!(
        outcome("looseEquality", lit("1"), SiteArg::Bool(true)),
        both
    );
    assert_eq!(outcome("looseEquality", object(&[]), object(&[])), both);
    assert_eq!(outcome("looseEquality", s.clone(), lit("a")), both);
    assert_eq!(outcome("strictEquality", lit("a"), SiteArg::Bool(true)), no);
    assert_eq!(outcome("strictEquality", n.clone(), u.clone()), no);
    assert_eq!(outcome("strictEquality", u.clone(), u), yes);
    assert_eq!(outcome("strictEquality", n.clone(), n), yes);
    assert_eq!(outcome("strictEquality", s.clone(), s), both);
    assert_eq!(
        outcome("strictEquality", SiteArg::Array(vec![]), lit("a")),
        both
    );
}
