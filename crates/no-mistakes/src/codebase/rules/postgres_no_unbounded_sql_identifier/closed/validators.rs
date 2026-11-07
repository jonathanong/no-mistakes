use super::*;
use crate::codebase::dependencies::extract::InvocationKind;

pub(super) fn matches(
    graph: &DepGraph,
    opts: &CompiledOptions,
    file: &Path,
    callee: &str,
    argument: usize,
    offset: u32,
) -> bool {
    let sites = graph.call_sites_in_file(file);
    let start = sites.partition_point(|site| site.offset < offset);
    let sites = &sites[start..];
    opts.identifier_validators.iter().any(|validator| {
        if validator.argument != argument {
            return false;
        }
        let Some(module) = &validator.module else {
            return validator.name == callee;
        };
        sites
            .iter()
            .take_while(|site| site.offset == offset)
            .filter(|site| site.invocation == InvocationKind::Call)
            .filter(|site| site.source_callee == callee)
            .any(|site| graph.matches_module_export(module, &validator.name, site))
    })
}

#[cfg(test)]
#[path = "validators_tests.rs"]
mod tests;
