use crate::codebase::ts_source::facts::TsFactPlan;

pub fn extract_imports_from_program<'a>(program: &Program<'a>) -> Vec<ExtractedImport> {
    extract_import_facts_from_program(program).imports
}

pub fn extract_import_facts_from_program<'a>(program: &Program<'a>) -> ImportFacts {
    extract_import_facts_from_program_with_source(program, "")
}

pub fn extract_import_facts_from_program_with_source<'a>(
    program: &Program<'a>,
    source: &str,
) -> ImportFacts {
    extract_import_facts_from_program_with_source_and_resource_roots(program, source, true)
}

pub(crate) fn extract_import_facts_from_program_with_source_and_resource_roots(
    program: &Program<'_>,
    source: &str,
    collect_resource_roots: bool,
) -> ImportFacts {
    extract_import_facts_from_program_with_source_and_demands(
        program,
        source,
        collect_resource_roots,
        false,
        false,
    )
}

pub(crate) fn extract_import_facts_from_program_with_source_and_demands(
    program: &Program<'_>,
    source: &str,
    collect_resource_roots: bool,
    collect_per_item_calls: bool,
    collect_value_flow: bool,
) -> ImportFacts {
    extract_import_facts_with_plan_and_rhs_probe(
        program,
        source,
        TsFactPlan {
            resources: collect_resource_roots,
            per_item_calls: collect_per_item_calls,
            value_flow: collect_value_flow,
            identifier_provenance: collect_value_flow,
            ..Default::default()
        },
        #[cfg(feature = "test-instrumentation")]
        None,
    )
}

pub(crate) fn extract_import_facts_with_plan(
    program: &Program<'_>,
    source: &str,
    plan: TsFactPlan,
) -> ImportFacts {
    extract_import_facts_with_plan_and_rhs_probe(
        program,
        source,
        TsFactPlan {
            value_flow: plan.needs_value_flow(),
            ..plan
        },
        #[cfg(feature = "test-instrumentation")]
        None,
    )
}

#[cfg(all(test, feature = "test-instrumentation"))]
fn extract_import_facts_with_rhs_probe(
    program: &Program<'_>,
    source: &str,
    collect_resource_roots: bool,
    collect_per_item_calls: bool,
    collect_value_flow: bool,
    collect_call_eligibility: bool,
    probe: Option<
        std::rc::Rc<
            std::cell::RefCell<extract_value_flow::inert_receiver_rhs::tests_support::Probe>,
        >,
    >,
) -> ImportFacts {
    extract_import_facts_with_plan_and_rhs_probe(
        program,
        source,
        TsFactPlan {
            resources: collect_resource_roots,
            per_item_calls: collect_per_item_calls,
            value_flow: collect_value_flow,
            identifier_provenance: collect_value_flow,
            call_eligibility: collect_call_eligibility,
            ..Default::default()
        },
        probe,
    )
}
