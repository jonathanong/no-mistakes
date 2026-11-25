#[derive(Debug, Clone, Copy, Default)]
pub struct ExtractDemands {
    pub resources: bool,
    pub per_item_calls: bool,
    pub value_flow: bool,
    pub identifier_provenance: bool,
    pub call_eligibility: bool,
    pub scoped_transaction_handles: bool,
}

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

pub fn extract_import_facts_from_program_with_source_and_resource_roots(
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

pub fn extract_import_facts_from_program_with_source_and_demands(
    program: &Program<'_>,
    source: &str,
    collect_resource_roots: bool,
    collect_per_item_calls: bool,
    collect_value_flow: bool,
) -> ImportFacts {
    extract_import_facts_with_plan_and_rhs_probe(
        program,
        source,
        ExtractDemands {
            resources: collect_resource_roots,
            per_item_calls: collect_per_item_calls,
            value_flow: collect_value_flow,
            identifier_provenance: collect_value_flow,
            ..Default::default()
        },
        None,
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
        ExtractDemands {
            resources: collect_resource_roots,
            per_item_calls: collect_per_item_calls,
            value_flow: collect_value_flow,
            identifier_provenance: collect_value_flow,
            call_eligibility: collect_call_eligibility,
            scoped_transaction_handles: false,
        },
        None,
        probe,
    )
}

pub fn extract_import_facts_with_demands_and_forwarders(
    program: &Program<'_>,
    source: &str,
    demands: ExtractDemands,
    forwarders: std::sync::Arc<FxHashSet<u32>>,
) -> ImportFacts {
    extract_import_facts_with_plan_and_rhs_probe(
        program,
        source,
        demands,
        Some(forwarders),
        #[cfg(feature = "test-instrumentation")]
        None,
    )
}
