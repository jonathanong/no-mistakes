use super::{exec, lit, load, object};
use crate::codebase::postgres::embedded::helpers::SiteArg;

#[test]
fn deferred_defaults_must_throw_for_unmatched_and_unknown_selector_values() {
    let programs = load("switch-boundaries.mts");
    for name in ["deferredValue", "unknownDefault"] {
        assert_eq!(
            exec(&programs, name, &[lit("a")]),
            Some(vec!["sql:SELECT 1".into()]),
            "{name}"
        );
        for selector in [lit("missing"), SiteArg::Scalar, SiteArg::Bool(true)] {
            assert_eq!(exec(&programs, name, &[selector]), None, "{name}");
        }
    }
    for name in ["throwingBranches", "inertPrefix"] {
        assert_eq!(
            exec(&programs, name, &[SiteArg::Scalar]),
            Some(vec!["sql:SELECT 1".into()]),
            "{name}"
        );
        assert_eq!(
            exec(&programs, name, &[lit("missing")]),
            Some(vec![]),
            "{name}"
        );
    }
}

#[test]
fn selector_and_arm_evaluation_failures_do_not_publish_partial_sql() {
    let programs = load("switch-boundaries.mts");
    assert_eq!(
        exec(
            &programs,
            "memberSelector",
            &[object(&[("kind", lit("a"))])]
        ),
        Some(vec!["sql:SELECT 1".into()])
    );
    let actual = vec![
        (
            "null-selector",
            exec(&programs, "memberSelector", &[SiteArg::Null]),
        ),
        (
            "unreadable-selector",
            exec(&programs, "memberSelector", &[SiteArg::Bool(true)]),
        ),
        (
            "known-failed-arm",
            exec(&programs, "failedArm", &[lit("a")]),
        ),
        (
            "unknown-failed-arm",
            exec(&programs, "failedArm", &[SiteArg::Scalar]),
        ),
    ];
    assert_eq!(
        actual,
        vec![
            ("null-selector", Some(vec![])),
            ("unreadable-selector", None),
            ("known-failed-arm", None),
            ("unknown-failed-arm", None),
        ]
    );
}

#[test]
fn unproven_prefix_effects_and_valueless_returns_do_not_gain_throw_proofs() {
    let programs = load("switch-boundaries.mts");
    let actual: Vec<_> = [
        "mutablePrefix",
        "multiplePrefix",
        "destructuredPrefix",
        "computedPrefix",
        "expressionPrefix",
        "valuelessDefault",
        "valuelessArm",
    ]
    .into_iter()
    .map(|name| (name, exec(&programs, name, &[lit("a")])))
    .collect();
    let expected = actual
        .iter()
        .map(|(name, _)| (*name, None))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
