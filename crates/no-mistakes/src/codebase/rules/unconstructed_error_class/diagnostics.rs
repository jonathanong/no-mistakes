use super::{
    analysis::{is_declaration_file, unresolved_constructor},
    RULE_ID,
};
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::RuleDef;
use std::path::Path;

pub(super) fn uncertain_class(
    mut finding: RuleFinding,
    application: &RuleDef,
    unknown_base: bool,
    unknown_constructor: bool,
) -> RuleFinding {
    let name = finding.target.as_deref().unwrap_or("default");
    let detail = if unknown_base {
        format!("Cannot determine whether exported class `{name}` is an error class because its base is outside the analyzed graph. Use an analyzable local error base or include the base's repository source, then rerun the check.")
    } else if unknown_constructor {
        format!("Cannot prove whether exported error class `{name}` is constructed because an unresolved constructor occurs in its lexical class body. Replace the dynamic constructor with a statically named class to establish construction, then rerun the check.")
    } else {
        format!("Cannot prove whether exported error class `{name}` is constructed because its namespace escapes static analysis. Replace namespace aliases or dynamic accesses with direct class construction or inheritance, then rerun the check.")
    };
    finding.message = message(application, &detail);
    finding.target = Some("unanalyzable".to_string());
    finding
}

pub(super) fn unknown_constructs(
    root: &Path,
    application: &RuleDef,
    graph: &DepGraph,
    is_test: impl Fn(&Path) -> bool,
    is_reported: impl Fn(&Path) -> bool,
) -> Vec<RuleFinding> {
    graph.resolved_call_sites().iter().filter(|site| {
        site.invocation == InvocationKind::Construct && unresolved_constructor(graph, site)
            && !is_test(&site.file) && is_reported(&site.file)
            && !is_declaration_file(&site.file)
    }).map(|site| RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, &site.file),
        line: site.line.max(1) as usize,
        message: message(application, &format!("Cannot resolve {}, so error-class construction coverage is incomplete. Replace the dynamic constructor with a statically named class or an analyzable import, then rerun the check.", constructor_subject(&site.source_callee, graph.constructor_expression(&site.file, site.offset)))),
        import: None,
        target: Some("unanalyzable".to_string()),
    }).collect()
}

fn message(application: &RuleDef, detail: &str) -> String {
    application.message.as_deref().map_or_else(
        || detail.to_string(),
        |custom| format!("{custom}: {detail}"),
    )
}

fn constructor_subject(callee: &str, expression: Option<&str>) -> String {
    if callee == "<unknown>" {
        expression.map_or_else(
            || "a computed or conditional constructor expression".to_string(),
            |expression| format!("constructor expression `{expression}`"),
        )
    } else {
        format!("construction target `{callee}`")
    }
}
