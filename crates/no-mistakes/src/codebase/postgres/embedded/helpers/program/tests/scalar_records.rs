use super::{lit, load, SiteArg};

#[test]
fn fresh_scalar_records_keep_member_bind_proofs() {
    let programs = load("scalar-records.ts");
    let lifecycle = &programs["lifecycle"];
    assert_eq!(
        lifecycle.scalar_fields,
        ["aAt", "absent", "amount", "enabled", "label"]
    );
    assert!(lifecycle.scalar_member("aAt", &[SiteArg::Scalar, SiteArg::Scalar]));
    assert!(!lifecycle.scalar_member("missing", &[SiteArg::Scalar, SiteArg::Scalar]));
    for arg in [
        SiteArg::Scalar,
        lit("scalar"),
        SiteArg::Bool(true),
        SiteArg::Null,
        SiteArg::Undefined,
    ] {
        assert!(programs["arrow"].scalar_member("value", &[arg]));
    }
    for args in [
        vec![],
        vec![SiteArg::Builder],
        vec![SiteArg::Object(vec![])],
        vec![SiteArg::Array(vec![])],
    ] {
        assert!(!programs["arrow"].scalar_member("value", &args));
    }
}

#[test]
fn effectful_structural_or_non_record_helpers_never_prove_scalar_members() {
    let programs = load("scalar-records.ts");
    for name in [
        "notRecord",
        "spread",
        "prototype",
        "duplicate",
        "getter",
        "method",
        "builder",
        "defaulted",
        "global",
        "member",
        "called",
        "sqlField",
        "nested",
        "array",
        "extra",
        "badTest",
        "badYes",
        "badNo",
        "badLeft",
        "badRight",
        "unusedDefault",
        "unusedBuilder",
        "looseEquality",
    ] {
        assert!(
            programs
                .get(name)
                .is_none_or(|program| program.scalar_fields.is_empty()),
            "{name} must not prove a scalar record"
        );
    }
}
