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
    opts.identifier_validators.iter().any(|validator| {
        if validator.argument != argument { return false; }
        let Some(module) = &validator.module else { return validator.name == callee; };
        graph.call_sites_in_file(file).iter().filter(|site| site.offset == offset).any(|site| {
            let direct = matches!(&site.target, ResolvedCallTarget::ModuleExport { specifier, export_path, .. }
                if specifier == module && export_path == &validator.name);
            direct || site.target_node.as_ref().is_some_and(|target| graph.module_export_nodes(module, &validator.name).contains(target))
        })
    })
}
