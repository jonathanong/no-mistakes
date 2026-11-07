use super::*;
use crate::codebase::dependencies::graph::ResolvedCallTarget;

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
        if validator.argument != argument { return false; }
        let Some(module) = &validator.module else { return validator.name == callee; };
        sites.iter().take_while(|site| site.offset == offset).any(|site| {
            let direct = matches!(&site.target, ResolvedCallTarget::ModuleExport { specifier, export_path, .. }
                if specifier == module && export_path == &validator.name);
            direct || site.target_node.as_ref().is_some_and(|target| graph.module_export_nodes(module, &validator.name).contains(target))
        })
    })
}
