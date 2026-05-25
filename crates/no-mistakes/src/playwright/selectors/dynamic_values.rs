pub(super) mod collect;
pub(super) mod cross_file;
#[cfg(test)]
mod tests;
mod visitor;

pub(super) use collect::collect_function_return_strings;
pub use collect::collect_string_leaves;

use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_span::Span;

pub(super) struct DynamicIdentifierValues {
    pub(super) name: String,
    pub(super) values: Vec<String>,
    pub(super) scope: Span,
}

pub(super) fn resolve_dynamic_identifier(
    name: &str,
    span: Span,
    dynamic_values: &[DynamicIdentifierValues],
) -> Vec<String> {
    let candidates: Vec<_> = dynamic_values
        .iter()
        .filter(|dv| dv.name == name && dv.scope.start <= span.start && span.end <= dv.scope.end)
        .collect();
    let Some(min_size) = candidates
        .iter()
        .map(|dv| dv.scope.end - dv.scope.start)
        .min()
    else {
        return vec![];
    };
    candidates
        .into_iter()
        .filter(|dv| dv.scope.end - dv.scope.start == min_size)
        .flat_map(|dv| dv.values.iter().cloned())
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn collect_dynamic_identifier_values(
    program: &Program<'_>,
    source: &str,
) -> Vec<DynamicIdentifierValues> {
    collect_dynamic_identifier_values_for_file(program, source, None)
}

pub(super) fn collect_dynamic_identifier_values_with_file(
    program: &Program<'_>,
    source: &str,
    file: &std::path::Path,
) -> Vec<DynamicIdentifierValues> {
    collect_dynamic_identifier_values_for_file(program, source, Some(file))
}

fn collect_dynamic_identifier_values_for_file(
    program: &Program<'_>,
    _source: &str,
    file: Option<&std::path::Path>,
) -> Vec<DynamicIdentifierValues> {
    let mut v = visitor::DynamicValuesVisitor::new();
    v.visit_program(program);

    let mut resolved = Vec::new();
    for mut entry in v.collected {
        let mut new_values = Vec::new();
        let mut had_sentinel = false;
        for value in &entry.values {
            if let Some(fn_name) = value.strip_prefix("__call__") {
                had_sentinel = true;
                let ret_vals = collect_function_return_strings(fn_name, program);
                if !ret_vals.is_empty() {
                    new_values.extend(ret_vals);
                } else if let Some(f) = file {
                    new_values.extend(cross_file::resolve_imported_values(fn_name, program, f));
                }
            } else if let Some(obj_name) = value.strip_prefix("__obj__") {
                had_sentinel = true;
                if let Some(f) = file {
                    new_values.extend(cross_file::resolve_imported_values(obj_name, program, f));
                }
            } else {
                new_values.push(value.clone());
            }
        }
        if had_sentinel {
            entry.values = new_values;
        }
        if !entry.values.is_empty() {
            resolved.push(entry);
        }
    }
    resolved
}
