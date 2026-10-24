use super::*;
pub(super) struct Contract {
    pub from: NodeId,
    pub to: NodeId,
    pub via: config::Import,
}
pub(super) fn resolve(
    root: &Path,
    options: &Options,
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
) -> Result<Vec<Contract>> {
    options
        .ownership
        .iter()
        .map(|contract| {
            Ok(Contract {
                from: contract.producer.resolve(root, graph, facts)?,
                to: contract.owner.resolve(root, graph, facts)?,
                via: contract.via.clone(),
            })
        })
        .collect()
}
/// One typed relationship per real configured dispatch; never a second call graph.
pub(crate) fn ownership_edges(
    root: &Path,
    applications: &[RuleDef],
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
) -> Result<Vec<(NodeId, NodeId, EdgeKind)>> {
    let mut edges = Vec::new();
    for application in applications {
        let options: Options = application.try_rule_options()?;
        options.validate()?;
        for contract in resolve(root, &options, graph, facts)? {
            if generator(&contract.from, facts)
                || generator(&contract.to, facts)
                || unavailable(contract.from.as_file().expect("callable file"), facts)
                || unavailable(contract.to.as_file().expect("callable file"), facts)
            {
                continue;
            }
            if graph
                .call_sites_in_file(contract.from.as_file().expect("callable file"))
                .iter()
                .any(|site| {
                    source(site) == contract.from
                        && eligible(site, facts)
                        && !site.target_nodes().any(|target| generator(target, facts))
                        && contract.via.matches(site)
                })
            {
                edges.push((contract.from, contract.to, EdgeKind::AssertionOwner));
            }
        }
    }
    Ok(edges)
}
