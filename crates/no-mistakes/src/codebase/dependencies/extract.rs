pub use no_mistakes_ts_facts::extract::*;

use crate::codebase::ts_source::facts::TsFactPlan;
use crate::fx::FxHashSet;
use oxc_ast::ast::Program;

pub(crate) fn extract_import_facts_with_forwarders(
    program: &Program<'_>,
    source: &str,
    plan: TsFactPlan,
    forwarders: std::sync::Arc<FxHashSet<u32>>,
) -> ImportFacts {
    let demands = ExtractDemands {
        resources: plan.resources,
        per_item_calls: plan.per_item_calls,
        value_flow: plan.needs_value_flow(),
        identifier_provenance: plan.identifier_provenance,
        call_eligibility: plan.call_eligibility,
        scoped_transaction_handles: plan.scoped_transaction_handles,
    };
    extract_import_facts_with_demands_and_forwarders(program, source, demands, forwarders)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "extract/tests/no_object_mutation_literal.rs"]
mod no_object_mutation_literal;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "extract_value_flow/identifier_provenance_demand_tests.rs"]
mod identifier_provenance_demand_tests;
