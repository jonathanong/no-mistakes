include!("extract_entrypoints_demands.rs");

fn extract_import_facts_with_plan_and_rhs_probe(
    program: &Program<'_>,
    source: &str,
    plan: TsFactPlan,
    forwarders: Option<std::sync::Arc<FxHashSet<u32>>>,
    #[cfg(feature = "test-instrumentation")] probe: Option<
        std::rc::Rc<
            std::cell::RefCell<extract_value_flow::inert_receiver_rhs::tests_support::Probe>,
        >,
    >,
) -> ImportFacts {
    let collector = crate::diagnostics::hot_files::trace_current(
        crate::diagnostics::hot_files::FilePass::ValueFlowExtract,
        source.len() as u64,
        || {
            let mut collector = ImportCollector {
                transparent_receiver_writes: forwarders.unwrap_or_else(|| crate::codebase::ts_source::module_facts::arguments::transparent_receiver_writes_from_program(program)),
                collect_per_item_calls: plan.per_item_calls,
                collect_value_flow: plan.value_flow,
                collect_identifier_provenance: plan.identifier_provenance,
                collect_call_eligibility: plan.call_eligibility,
                line_starts: import_line_starts(source),
                collect_resource_roots: plan.resources,
                sloppy_script: program.source_type.is_javascript()
                    && !program.source_type.is_strict()
                    && !program.has_use_strict_directive(),
                ..ImportCollector::default()
            };
            #[cfg(feature = "test-instrumentation")]
            {
                collector.fresh_receiver_effects.rhs_probe = probe;
            }
            // Establish the file lexical environment before visiting expressions so a
            // later declaration cannot make an earlier global-looking call resolve to
            // a built-in. This is collection from the already parsed Program, not a
            // second source pass.
            collector.local_stack.push(fx_set());
            // Annotation shadow lookup reserves index zero for the program type scope.
            collector.type_local_stack.push(fx_set());
            collector.lexical_scope_ids.push(0);
            collector.lexical_scope_parents.insert(0, None);
            collector.next_lexical_scope_id = 1;
            predeclare_program_value_bindings(&mut collector, program);
            let local_type_names = local_type_declaration_names(program);
            collector
                .exported_functions
                .extend(later_named_value_exports(program, &local_type_names));
            collector
                .exported_functions
                .extend(later_default_export_value_names(program));
            collector
                .later_exported_type_names
                .extend(later_named_type_exports(program, &local_type_names));
            collector.visit_program(program);
            collector
        },
    );
    crate::diagnostics::hot_files::trace_current(
        crate::diagnostics::hot_files::FilePass::ValueFlowFinalize,
        source.len() as u64,
        || finish_import_facts(collector, program),
    )
}

include!("extract_entrypoints_finish.rs");
include!("extract_entrypoints_test_modules.rs");
