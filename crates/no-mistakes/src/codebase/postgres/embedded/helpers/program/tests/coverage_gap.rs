use crate::codebase::postgres::embedded::helpers::SiteArg;

#[test]
fn coverage_gap_lowered_programs() {
    let programs = super::load("gap-branches.ts");
    for name in [
        "elseRedeclare",
        "forOfRedeclare",
        "emptyDefault",
        "bareConst",
    ] {
        assert!(!programs.contains_key(name), "{name} lowered");
    }
    assert_eq!(
        super::one(&programs, "known", &[super::lit("a")]),
        "sql:SELECT 1"
    );
    let scalar = SiteArg::Scalar;
    super::fails(&programs, "nestedFunction", std::slice::from_ref(&scalar));
    super::fails(&programs, "andLeft", &[SiteArg::Bool(true)]);
    super::fails(&programs, "andRight", &[SiteArg::Bool(true)]);
    super::fails(&programs, "nullishLeft", &[]);
    super::fails(&programs, "nullishRight", &[]);
    assert_eq!(
        super::one(&programs, "spreadUnknown", std::slice::from_ref(&scalar)),
        "sql:SELECT 1"
    );
    super::fails(&programs, "openBag", &[SiteArg::Builder]);
    let both = [
        "sql:SELECT 1 AND a = 1".to_string(),
        "sql:SELECT 1".to_string(),
    ];
    assert_eq!(
        super::exec(&programs, "undefKey", &[SiteArg::Scalar]).expect("undefKey"),
        both
    );
    assert_eq!(
        super::exec(
            &programs,
            "eqAnd",
            &[SiteArg::Bool(true), SiteArg::Bool(false), SiteArg::Scalar,],
        )
        .expect("eqAnd"),
        both
    );
    assert_eq!(
        super::one(&programs, "defaultSql", &[super::lit("a")]),
        "str:no"
    );
    super::fails(&programs, "defaultSql", &[super::lit("b")]);
    super::fails(&programs, "defaultSql", std::slice::from_ref(&scalar));
    super::fails(&programs, "boolDefault", &[SiteArg::Bool(true)]);
    assert_eq!(
        super::one(&programs, "shortCall", &[SiteArg::Bool(false)]),
        "sql:SELECT 1"
    );
    super::fails(&programs, "shortCall", &[SiteArg::Bool(true)]);
    assert!(programs.contains_key("notBuilder"));
    super::fails(&programs, "notBuilder", &[super::lit("name")]);
}
