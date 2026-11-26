use super::CollectedOrigins;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

mod csp_scope;
mod csp_strings;
mod jsx;
mod registries;
mod url;
mod visit;

use registries::ConstArrays;

pub(crate) fn collect_for_plan(
    path: &std::path::Path,
    source: &str,
    program: &Program<'_>,
    plan: &crate::codebase::check_facts::CheckFactPlan,
) -> CollectedOrigins {
    if plan.csp_allowed_origins_paths.contains(path) {
        collect(program, source, path)
    } else {
        CollectedOrigins::default()
    }
}

fn collect(program: &Program<'_>, source: &str, path: &Path) -> CollectedOrigins {
    let arrays = Arc::new(ConstArrays::build(program, path));
    let mut visitor = OriginVisitor {
        source,
        collected: CollectedOrigins::default(),
        csp_depth: 0,
        in_csp_fn: false,
        resolved_bindings: HashSet::new(),
        arrays: Arc::clone(&arrays),
    };
    visitor.visit_program(program);
    visitor.collected.const_string_exports = arrays.exported_strings();
    visitor.collected
}

struct OriginVisitor<'a> {
    source: &'a str,
    collected: CollectedOrigins,
    csp_depth: usize,
    in_csp_fn: bool,
    resolved_bindings: HashSet<String>,
    arrays: Arc<ConstArrays>,
}
