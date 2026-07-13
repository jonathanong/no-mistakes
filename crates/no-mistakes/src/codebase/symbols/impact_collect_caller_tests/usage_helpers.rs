use super::*;

#[test]
fn dynamic_usage_helpers_ignore_non_module_and_malformed_bindings() {
    assert!(dynamic_module_bindings("const utils = await import('./utils.mts');").contains("utils"));
    assert!(dynamic_module_bindings("const readDate = require('./utils.mts').parseDate;").is_empty());
    assert!(destructured_symbol_aliases("const { parseDate = await import('./utils.mts');", "parseDate").is_empty());
    assert!(member_assignment_alias("exports.value = utils.parseDate;", "parseDate").is_empty());
    assert!(member_assignment_alias("const readDate = utils.other;", "parseDate").is_empty());
    assert!(dynamic_symbol_aliases_in_source("const utils = await import('./utils.mts');", "dates.parseDate").is_empty());
    assert!(source_contains_member_name("utils.parseDate(value)", "utils.parseDate"));
    assert!(!source_contains_member_name(
        "utils.parseDateOld(value)",
        "utils.parseDate"
    ));
    assert!(source_contains_call_name("pd(value)", "pd"));
    assert!(source_contains_call_name("pd (value)", "pd"));
    assert!(!source_contains_call_name("otherpd(value)", "pd"));
}

#[test]
fn local_callee_matching_accepts_namespace_members() {
    assert!(matches_local_callee(
        "dates.parseDate",
        &BTreeSet::from(["dates".to_string()])
    ));
    assert!(matches_local_callee(
        "parseDate",
        &BTreeSet::from(["parseDate".to_string()])
    ));
    assert!(!matches_local_callee(
        "updatedDates.parseDate",
        &BTreeSet::from(["dates".to_string()])
    ));
}
