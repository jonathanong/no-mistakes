mod bodies;
mod body;
mod call;
mod collect;
mod collection;
mod exports;
mod identifier_clauses;
mod parameter;
mod positions;
mod programs;
pub(crate) mod provenance;
pub(crate) mod reassigned;
mod shadows;
#[cfg(test)]
mod tests;
use super::super::super::helpers::template::TemplateHelper;
use super::super::super::options::TrustedSqlTag;
use super::super::super::tags::SqlTagNames;
use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{
    source_positions, EmbeddedSqlKind, EmbeddedSqlSourcePosition,
};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashMap;
use exports::{template_helpers, Callables};
use oxc_ast::ast::{CallExpression, Expression, FormalParameters, FunctionBody, Program};
use reassigned::ReassignedNames;
pub(crate) use shadows::looks_like_tag_implementation;
use shadows::TagShadows;
use std::collections::HashMap;
pub(super) const MAX_RESOLVE_DEPTH: u8 = 8;
pub(crate) struct LocalFunctions {
    resolved: FxHashMap<String, provenance::RecoveredSql>,
    guarded_identifiers: FxHashMap<String, Vec<String>>,
    templates: FxHashMap<String, TemplateHelper>,
    parameter_builders: HashMap<String, parameter::ParameterBuilder>,
    tag_shadows: TagShadows,
    programs: FxHashMap<String, std::sync::Arc<super::super::super::helpers::program::Program>>,
    declared_elsewhere: crate::fx::FxHashSet<String>,
}
pub(super) struct Resolvable<'a> {
    params: &'a FormalParameters<'a>,
    body: &'a FunctionBody<'a>,
}
impl LocalFunctions {
    pub(crate) fn collect_heads(program: &Program<'_>, trusted_sql_tags: &[TrustedSqlTag]) -> Self {
        Self {
            resolved: Default::default(),
            guarded_identifiers: Default::default(),
            templates: Default::default(),
            parameter_builders: Default::default(),
            tag_shadows: TagShadows::collect(
                program,
                trusted_sql_tags,
                &super::super::super::bindings::sql_statement_type_bindings(program),
            ),
            programs: Default::default(),
            declared_elsewhere: bodies::collect(program),
        }
    }
    pub(crate) fn collect_heads_with_raw(
        program: &Program<'_>,
        trusted: &[TrustedSqlTag],
        raw: &[TrustedSqlTag],
    ) -> Self {
        let mut value = Self::collect_heads(program, trusted);
        value.tag_shadows.configure_raw(
            program,
            raw,
            &super::super::super::bindings::sql_statement_type_bindings(program),
        );
        value
    }
    pub(crate) fn has_static_helper(&self, name: &str) -> bool {
        self.resolved.contains_key(name) || self.templates.contains_key(name)
    }

    pub(crate) fn collect_with_raw(
        program: &Program<'_>,
        source: &str,
        trusted: &[TrustedSqlTag],
        raw: &[TrustedSqlTag],
    ) -> Self {
        collection::collect(program, source, trusted, raw)
    }
    pub(crate) fn is_parameter_builder(&self, name: &str) -> bool {
        self.parameter_builders.contains_key(name)
    }

    pub(in crate::codebase::postgres::embedded::walk) fn parameter_builder(
        &self,
        name: &str,
    ) -> Option<&parameter::ParameterBuilder> {
        self.parameter_builders.get(name)
    }

    pub(crate) fn is_tag_shadowed(&self, name: &str) -> bool {
        self.tag_shadows.contains(name)
    }

    pub(crate) fn imported_sql_tags(&self) -> &SqlTagNames {
        self.tag_shadows.imported_tags()
    }
}
pub(super) fn shadows_param(resolvable: &Resolvable<'_>, name: &str) -> bool {
    let mut shadows = false;
    let mut check = |bound: &str| shadows |= bound == name;
    for param in &resolvable.params.items {
        super::for_each_bound_name(&param.pattern, &mut check);
    }
    if let Some(rest) = &resolvable.params.rest {
        super::for_each_bound_name(&rest.rest.argument, &mut check);
    }
    shadows
}

fn resolve_named(
    name: &str,
    source: &str,
    depth: u8,
    callables: &Callables<'_, '_>,
    resolving: &mut Vec<String>,
    tag_shadows: &TagShadows,
) -> Option<provenance::RecoveredSql> {
    if resolving.iter().any(|seen| seen == name) {
        return None;
    }
    let resolvable = callables.raw.get(name)?;
    resolving.push(name.to_string());
    let mut lookup = |call: &CallExpression<'_>, callee: &str, depth: u8| {
        if shadows_param(resolvable, callee) {
            return None;
        }
        if let Some(template) = callables.templates.get(callee) {
            return call::apply_template(template, call, source);
        }
        resolve_named(callee, source, depth, callables, resolving, tag_shadows)
    };
    let mut is_shadowed = |tag: &str| shadows_param(resolvable, tag) || tag_shadows.contains(tag);
    let text = body::resolve(
        resolvable,
        source,
        depth,
        &mut lookup,
        &mut is_shadowed,
        tag_shadows.imported_tags(),
    );
    resolving.pop();
    text
}
