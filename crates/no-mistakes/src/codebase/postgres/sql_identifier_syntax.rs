//! Low-effort SQL inputs from the request's parsed program and semantic bindings.
//! This does not evaluate JavaScript or infer provenance through executor/value aliases.
pub(crate) mod literal_projection;
use super::EmbeddedSqlOptions;
use crate::codebase::ts_source::{facts::TsFileFacts, unwrap_ts_wrappers};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, Program, VariableDeclaration};
use oxc_ast_visit::{walk, Visit};

type Span = (u32, u32);
fn span(value: oxc_span::Span) -> Span {
    (value.start, value.end)
}

#[derive(Clone, Debug)]
pub(crate) enum Input {
    Sql {
        text: String,
        source_span: Span,
        holes: Vec<(String, Option<String>)>,
        parameterized: Option<usize>,
        positions: Vec<super::EmbeddedSqlSourcePosition>,
    },
    Recovered(std::sync::Arc<helper_recovery::Recovered>),
    Reference(Span),
    Unknown,
}
#[derive(Clone, Debug)]
pub(crate) struct SqlCall {
    pub span: Span,
    pub line: u32,
    pub input: Input,
}
pub(crate) type OffsetLiteral = literal_projection::ProjectedLiteral<super::SqlOffsetFact>;
pub(crate) type TransformedLiteral =
    literal_projection::ProjectedLiteral<super::statements::SqlTransformedColumnUseFact>;
#[derive(Default)]
pub(crate) struct FileFacts {
    pub low_applications: Vec<(EmbeddedSqlOptions, String, usize)>,
    pub profiles: Vec<(EmbeddedSqlOptions, Vec<SqlCall>)>,
    pub recovery: helper_recovery::FileModels,
    pub application_calls:
        std::collections::BTreeMap<helper_recovery::ApplicationKey, Vec<SqlCall>>,
    pub transformed_literals: std::sync::OnceLock<
        std::collections::BTreeMap<
            (Option<helper_recovery::ApplicationKey>, Span),
            TransformedLiteral,
        >,
    >,
    pub offset_literals: std::sync::OnceLock<
        std::collections::BTreeMap<(Option<helper_recovery::ApplicationKey>, Span), OffsetLiteral>,
    >,
    /// One SQL parse per recovered alternative, shared by low policy projections.
    pub recovered_statements: std::sync::OnceLock<literal_projection::RecoveredStatements>,
}
struct Call {
    span: Span,
    callee: Option<(Span, Vec<String>)>,
    input: Input,
}
#[derive(Default)]
struct Syntax<'source> {
    source: &'source str,
    recovery_bindings: helper_recovery::lower::Bindings,
    recovery: helper_recovery::FileModels,
    template_binds: template_binds::TemplateBinds,
    safe_parameterized: FxHashSet<Span>,
    metadata: metadata::Metadata,
    targets: FxHashSet<Span>,
    namespace_targets: FxHashMap<Span, FxHashSet<String>>,
    typed_targets: FxHashSet<Span>,
    loose_query: bool,
    calls: Vec<Call>,
    constants: FxHashMap<Span, Input>,
    env_references: FxHashMap<String, Span>,
    namespace_members: FxHashMap<Span, String>,
    aliases: FxHashMap<Span, usize>,
    alias_initializers: FxHashMap<Span, usize>,
}
fn chain(expression: &Expression<'_>) -> Option<(Span, Vec<String>)> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(id) => Some((span(id.span), vec![id.name.to_string()])),
        Expression::StaticMemberExpression(value) => {
            let (root, mut parts) = chain(&value.object)?;
            parts.push(value.property.name.to_string());
            Some((root, parts))
        }
        Expression::ComputedMemberExpression(value) => {
            let Expression::StringLiteral(key) = unwrap_ts_wrappers(&value.expression) else {
                return None;
            };
            let (root, mut parts) = chain(&value.object)?;
            parts.push(key.value.to_string());
            Some((root, parts))
        }
        _ => None,
    }
}
mod constants;
pub(crate) mod helper_recovery;
mod input;
mod template_binds;
use input::input;
mod metadata;
mod syntax_walk;

struct Constants<'s, 'source> {
    wanted: &'s FxHashSet<Span>,
    syntax: &'s mut Syntax<'source>,
}

mod selection;
mod typed;
use selection::Selection;
mod project;
pub(crate) use project::collect;
pub(crate) use project::collect_with_types;

mod invalid;
pub(crate) use invalid::collect_invalid;

pub(crate) mod plan;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod template_bind_tests;

#[cfg(test)]
mod metadata_tests;

#[cfg(test)]
mod runtime_span_tests;

#[cfg(test)]
mod escape_site_tests;

#[cfg(test)]
mod low_max_bind_parity_tests;

#[cfg(test)]
mod typed_computed_parameter_tests;
