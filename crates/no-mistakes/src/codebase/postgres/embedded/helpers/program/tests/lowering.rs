//! Which helpers lower to a program, and how their parameters are typed.

use super::{exec, lit, load};
use crate::codebase::postgres::embedded::helpers::SiteArg;

#[test]
fn supported_declarations_lower() {
    let programs = load("lower.ts");
    for name in [
        "plain",
        "arrowBlock",
        "arrowExpression",
        "functionExpression",
        "defaulted",
        "callsOnly",
        "forOf",
        "nestedFunctionsAreOpaque",
        "ifElseIf",
        "withBuilderTypes",
        "loopWithPattern",
        "withUninitialized",
    ] {
        assert!(programs.contains_key(name), "{name} should lower");
    }
    assert_eq!(
        exec(&programs, "arrowExpression", &[SiteArg::Scalar]),
        Some(vec!["sql:SELECT $1".to_string()])
    );
    assert_eq!(
        exec(&programs, "defaulted", &[SiteArg::Scalar]),
        Some(vec!["sql:SELECT $1".to_string()])
    );
    assert_eq!(
        exec(
            &programs,
            "ifElseIf",
            &[SiteArg::Bool(false), SiteArg::Bool(true)]
        ),
        Some(vec!["sql:SELECT 2".to_string()])
    );
}

#[test]
fn unmodelable_declarations_do_not_lower() {
    let programs = load("lower.ts");
    for name in [
        "destructuredParameter",
        "restParameter",
        "duplicateParameter",
        "redeclaredInBlock",
        "loopVariableShadowsParameter",
        "asynchronous",
        "generator",
        "noSql",
        "reassigned",
    ] {
        assert!(!programs.contains_key(name), "{name} should not lower");
    }
}

#[test]
fn statements_outside_the_model_fail_when_reached() {
    let programs = load("lower.ts");
    // A nested function is called through a local name, which is never the
    // module's helper.
    assert_eq!(exec(&programs, "nestedFunctionsAreOpaque", &[]), None);
    // A loop over a pattern, a `var` loop and a `break` are all opaque.
    assert_eq!(
        exec(
            &programs,
            "loopWithPattern",
            &[SiteArg::Array(vec![lit("a")])]
        ),
        None
    );
    // `let q;` has no initializer.
    assert_eq!(exec(&programs, "withUninitialized", &[]), None);
}

#[test]
fn only_sqlstatement_annotated_parameters_take_a_builder() {
    let programs = load("lower.ts");
    let builder = SiteArg::Builder;
    let scalar = SiteArg::Scalar;
    assert!(exec(
        &programs,
        "withBuilderTypes",
        &[builder.clone(), scalar.clone(), scalar.clone()]
    )
    .is_some());
    // An array of builders and a string are not builder parameters.
    assert_eq!(
        exec(
            &programs,
            "withBuilderTypes",
            &[scalar.clone(), builder.clone(), scalar.clone()]
        ),
        None
    );
    assert_eq!(
        exec(
            &programs,
            "withBuilderTypes",
            &[scalar.clone(), scalar, builder]
        ),
        None
    );
}
