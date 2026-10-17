//! What the walk summarizes for importers of this file.

use super::*;
use crate::codebase::postgres::embedded::helpers::BuilderHelper;

fn summary(facts: &EmbeddedSqlFileFacts, name: &str) -> Option<HelperSummary> {
    facts.exported_helpers.get(name).cloned()
}

fn literals(values: &[&str]) -> ImportedSite {
    ImportedSite {
        local: "helper".into(),
        args: values
            .iter()
            .map(|value| SiteArg::Literal((*value).into()))
            .collect(),
    }
}

#[test]
fn only_literal_identifier_templates_are_summarized_as_templates() {
    let facts = extract("exports.ts");
    assert!(matches!(
        summary(&facts, "plain"),
        Some(HelperSummary::Static(_))
    ));
    for name in [
        "arrow",
        "repeated",
        "multiline",
        "withBlockBody",
        "visibleByDefault",
        "expression",
        "blockArrow",
    ] {
        assert!(
            matches!(summary(&facts, name), Some(HelperSummary::Template(_))),
            "{name}"
        );
    }
    for name in [
        "withDefault",
        "withRest",
        "withOptional",
        "withDestructuring",
        "withExpression",
        "withOtherName",
        "withStatement",
        "asynchronous",
        "withTag",
        "generated",
        "mismatched",
        "asyncArrow",
        "noInterpolation",
        "reassigned",
    ] {
        assert!(summary(&facts, name).is_none(), "{name} must stay opaque");
    }
}

#[test]
fn a_helper_body_can_call_a_literal_identifier_template() {
    let facts = extract("exports.ts");
    let origin = Path::new("/exports.ts");
    let filled = summary(&facts, "viaArrow")
        .expect("viaArrow")
        .fill(&literals(&[]), origin)
        .expect("static");
    assert_eq!(filled.sql, "(l.x IS NULL)");
}

#[test]
fn parameters_are_spliced_wherever_they_are_interpolated() {
    let facts = extract("exports.ts");
    let origin = Path::new("/exports.ts");
    let fill = |name: &str, args: &[&str]| {
        summary(&facts, name)
            .expect(name)
            .fill(&literals(args), origin)
            .map(|filled| filled.sql)
    };
    assert_eq!(fill("arrow", &["a.b"]).as_deref(), Some("(a.b IS NULL)"));
    assert_eq!(
        fill("repeated", &["x", "y"]).as_deref(),
        Some("x = y AND x IS NOT NULL")
    );
    // Arity must match exactly, and every argument must be a literal.
    assert_eq!(fill("arrow", &[]), None);
    assert_eq!(fill("arrow", &["a", "b"]), None);
    for hostile in [
        "multi\nline",
        "carriage\rreturn",
        "mark\u{10FFFE}",
        "marker\u{10FFFF}",
    ] {
        assert_eq!(fill("arrow", &[hostile]), None, "{hostile:?}");
    }
    let scalar = ImportedSite {
        local: "helper".into(),
        args: vec![SiteArg::Scalar],
    };
    let template = summary(&facts, "arrow").expect("arrow");
    assert!(template.fill(&scalar, origin).is_none());
}

#[test]
fn substituted_text_keeps_the_templates_physical_lines_and_module() {
    let facts = extract("exports.ts");
    let origin = Path::new("/exports.ts");
    let filled = summary(&facts, "multiline")
        .expect("multiline")
        .fill(&literals(&["a.b", "much.longer.column"]), origin)
        .expect("filled");
    assert_eq!(
        filled.sql,
        "(a.b IS NULL\n    OR much.longer.column IS NULL\n    AND EXISTS (SELECT 1 FROM t))"
    );
    let source = std::fs::read_to_string(fixture("exports.ts")).expect("fixture");
    let first = source
        .lines()
        .position(|line| line.contains("return `(${column} IS NULL"))
        .expect("line") as u32
        + 1;
    let line_of = |needle: &str| {
        let offset = filled.sql.find(needle).expect(needle);
        origin_at(
            &filled.positions,
            filled.line,
            coordinate(&filled.sql, offset),
        )
    };
    let module = Some(origin.to_path_buf());
    assert_eq!(line_of("(a.b"), (first, module.clone()));
    assert_eq!(line_of("OR much"), (first + 1, module.clone()));
    assert_eq!(line_of("much.longer"), (first + 1, module.clone()));
    assert_eq!(line_of("EXISTS"), (first + 2, module));
}

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
        args,
    };
    let filled = builder
        .fill(&call(vec![SiteArg::Builder, SiteArg::Scalar]), origin)
        .expect("fills");
    assert_eq!(filled.sql, format!(" AND tier = {INTERNAL_PLACEHOLDER}1"));
    // A builder helper that appends nothing contributes no text.
    let unchanged = summary(&facts, "returnsBuilder").expect("returnsBuilder");
    let nothing = unchanged
        .fill(&call(vec![SiteArg::Builder]), origin)
        .expect("fills");
    assert_eq!(nothing.sql, "");
    for wrong in [
        vec![SiteArg::Scalar, SiteArg::Builder],
        vec![SiteArg::Builder],
        vec![SiteArg::Literal("x".into()), SiteArg::Scalar],
        vec![SiteArg::Builder, SiteArg::Builder],
    ] {
        assert!(builder.fill(&call(wrong), origin).is_none());
    }
}
