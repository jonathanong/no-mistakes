/// Rejects a plan that asks for an edge kind resolved from TS facts when the
/// build has none to read.
fn require_core_edge_facts(plan: GraphBuildPlan, facts: Option<&dyn TsFactLookup>) -> Result<()> {
    if plan.route_imports && facts.is_none() {
        anyhow::bail!("TS import facts are required for route-import edges");
    }
    if plan.symbols && facts.is_none() {
        anyhow::bail!("TS symbol facts are required when symbol edges are requested");
    }
    if plan.calls && facts.is_none() {
        anyhow::bail!("TS call facts are required when call edges are requested");
    }
    if plan.extends && facts.is_none() {
        anyhow::bail!("TS call facts are required when extends edges are requested");
    }
    Ok(())
}
