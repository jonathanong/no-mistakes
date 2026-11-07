use super::*;
use crate::codebase::dependencies::extract::{InvocationKind, ValueCall};
use crate::codebase::rules::postgres_sql_cross_file::IdentifierReturns;

pub(in super::super) fn prepare(
    opts: &CompiledOptions,
    graph: &DepGraph,
    facts: &CheckFactMap,
) -> IdentifierReturns {
    let mut returns = IdentifierReturns::default();
    if opts.identifier_validators.is_empty() && opts.identifier_sources.is_empty() {
        return returns;
    }
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        for call in &ts.value_flow.calls {
            if call.bound {
                continue;
            }
            let validator = opts.identifier_validators.iter().any(|validator| {
                call.arguments.get(validator.argument).is_some()
                    && call
                        .spread_at
                        .is_none_or(|spread| spread > validator.argument)
                    && selected(
                        graph,
                        file,
                        call,
                        validator.module.as_deref(),
                        &validator.name,
                    )
            });
            let sources: Vec<_> = opts
                .identifier_sources
                .iter()
                .enumerate()
                .filter_map(|(index, source)| {
                    selected(graph, file, call, source.module.as_deref(), &source.name)
                        .then_some(index)
                })
                .collect();
            if validator || !sources.is_empty() {
                returns.insert(file.to_path_buf(), (call.offset, call.end), sources);
            }
        }
    }
    returns
}

fn selected(
    graph: &DepGraph,
    file: &Path,
    call: &ValueCall,
    module: Option<&str>,
    name: &str,
) -> bool {
    let Some(module) = module else {
        return call.source_callee == name;
    };
    let sites = graph.call_sites_in_file(file);
    let start = sites.partition_point(|site| site.offset < call.offset);
    sites[start..]
        .iter()
        .take_while(|site| site.offset == call.offset)
        .filter(|site| site.caller_id == call.context && site.invocation == InvocationKind::Call)
        .filter(|site| site.source_callee == call.source_callee)
        .any(|site| graph.matches_module_export(module, name, site))
}

pub(in super::super) fn proven(
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> bool {
    let Provenance::Reference {
        name,
        scope,
        offset,
        members,
    } = provenance
    else {
        return false;
    };
    bindings
        .value(&call.file, *scope, name, *offset)
        .is_some_and(|binding| {
            binding.immutable && bindings.identifier_value(&call.file, &binding.value, members)
        })
}
