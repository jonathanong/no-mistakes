use super::*;

#[test]
fn a_statement_builder_helper_fills_only_a_matching_builder_call() {
    let facts = extract("exports.ts");
    let origin = Path::new("/exports.ts");
    let Some(HelperSummary::Builder(BuilderHelper {
        parameter_index,
        parameter_count,
        ..
    })) = summary(&facts, "appendsBuilder")
    else {
        panic!("appendsBuilder must be a builder summary");
    };
    assert_eq!((parameter_index, parameter_count), (0, 2));
    let builder = summary(&facts, "appendsBuilder").expect("builder");
    let call = |args: Vec<SiteArg>| ImportedSite {
        local: "helper".into(),
        member: None,
        kind: crate::codebase::postgres::embedded::helpers::SiteKind::Sql,
        args,
        value_used: false,
        line: 1,
        constant: None,
    };
    let filled = builder
        .fill(
            &call(vec![SiteArg::Builder, SiteArg::Scalar]),
            origin,
            origin,
        )
        .expect("fills");
    assert_eq!(
        filled.sql,
        format!(
            " AND tier = {}1",
            crate::codebase::postgres::embedded::placeholders::INTERNAL_BIND
        )
    );
    // A builder helper that appends nothing contributes no text.
    let unchanged = summary(&facts, "returnsBuilder").expect("returnsBuilder");
    let nothing = unchanged
        .fill(&call(vec![SiteArg::Builder]), origin, origin)
        .expect("fills");
    assert_eq!(nothing.sql, "");
    for wrong in [
        vec![SiteArg::Scalar, SiteArg::Builder],
        vec![SiteArg::Builder],
        vec![SiteArg::Literal("x".into()), SiteArg::Scalar],
        vec![SiteArg::Builder, SiteArg::Builder],
    ] {
        assert!(builder.fill(&call(wrong), origin, origin).is_none());
    }
}
