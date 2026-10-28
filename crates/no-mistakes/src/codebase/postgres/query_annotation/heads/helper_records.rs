//! Fresh helper results projected through one immutable local binding.
use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionBody, ArrowFunctionExpression, Expression, Function, FunctionBody,
    ObjectPropertyKind, PropertyKind, Statement,
};
use oxc_ast_visit::{walk, Visit};
mod builders;
mod destructure;
mod source;
pub(super) use builders::{append_callee, builder_candidate, builder_local_proof, confirmed_reads};
pub(super) use destructure::{
    candidate as destructure_candidate, local_proof as destructure_local_proof,
};
pub(super) use source::source_proof;

pub(super) fn local_result_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    call: &super::syntax::Call,
) -> Option<Summary> {
    local_proof(syntax, bindings, call)
        .or_else(|| builder_local_proof(syntax, bindings, call))
        .or_else(|| destructure_local_proof(syntax, bindings, call))
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Summary {
    Plain(HeadProof),
    PlainLocal { reference: Span },
    Builder { proof: HeadProof, tag: Span },
    ReturnedLocal { reference: Span },
}
impl Summary {
    pub(super) fn proof(self) -> HeadProof {
        match self {
            Self::Plain(proof) | Self::Builder { proof, .. } => proof,
            Self::PlainLocal { .. } | Self::ReturnedLocal { .. } => HeadProof::Unproven,
        }
    }
    pub(super) fn tag(self) -> Option<Span> {
        match self {
            Self::Plain(_) | Self::PlainLocal { .. } => None,
            Self::Builder { tag, .. } => Some(tag),
            Self::ReturnedLocal { .. } => None,
        }
    }
}

pub(super) fn function(value: &Function<'_>) -> Option<Summary> {
    if value.r#async || value.generator {
        return None;
    }
    returned(value.body.as_ref()?)
}
pub(super) fn arrow(value: &ArrowFunctionExpression<'_>) -> Option<Summary> {
    if value.r#async {
        return None;
    }
    let ArrowFunctionBody::FunctionBody(body) = &value.body else {
        return None;
    };
    returned(body)
}
fn returned(body: &FunctionBody<'_>) -> Option<Summary> {
    let Statement::ReturnStatement(last) = body.statements.last()? else {
        return None;
    };
    let mut returns = Returns(0);
    returns.visit_function_body(body);
    if returns.0 != 1 {
        return None;
    }
    let value = unwrap_ts_wrappers(last.argument.as_ref()?);
    if let Expression::Identifier(id) = value {
        return Some(Summary::ReturnedLocal {
            reference: (id.span.start, id.span.end),
        });
    }
    if let Some(tag) = super::trusted_append::tag(value) {
        let head = super::syntax::head(value);
        let proof = match head {
            super::syntax::Head::AppendPrefixTag => HeadProof::Literal,
            super::syntax::Head::Text(value) if value.opaque => value.current,
            head => head.proof(false),
        };
        return Some(Summary::Builder { proof, tag });
    }
    let Expression::ObjectExpression(object) = value else {
        return None;
    };
    let mut text = None;
    let mut values = false;
    for entry in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = entry else {
            return None;
        };
        if property.computed || property.method || property.kind != PropertyKind::Init {
            return None;
        }
        let key = crate::ast::property_key_name(&property.key)?;
        match key.as_ref() {
            "text" if text.is_none() => {
                let value = unwrap_ts_wrappers(&property.value);
                text = Some(match value {
                    Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                        Summary::Plain(super::syntax::head(value).proof(false))
                    }
                    Expression::Identifier(id) => Summary::PlainLocal {
                        reference: (id.span.start, id.span.end),
                    },
                    _ => return None,
                });
            }
            "values" if !values => values = true,
            "text" | "values" | "__proto__" => return None,
            _ => {}
        }
    }
    text.filter(|_| values)
}
struct Returns(usize);
impl<'a> Visit<'a> for Returns {
    fn visit_function(&mut self, _: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
    fn visit_return_statement(&mut self, value: &oxc_ast::ast::ReturnStatement<'a>) {
        self.0 += 1;
        walk::walk_return_statement(self, value);
    }
}

pub(super) fn candidate<'a>(
    syntax: &Syntax,
    bindings: &'a Bindings<'_>,
    call: &super::syntax::Call,
) -> Option<&'a ModuleBinding> {
    if !syntax.record_result_calls.contains(&call.span) {
        return None;
    }
    let (use_span, call) = match call.head {
        super::syntax::Head::FieldReference(reference) => (reference, call.span),
        _ => return None,
    };
    let result = bindings.owner(use_span).filter(|owner| !owner.imported)?;
    let declaration = (result.span.start, result.span.end);
    let (callee, init_call) = *syntax.record_initializers.get(&declaration)?;
    if !bindings.declaration_safe(declaration)
        || bindings.exported(result)
        || syntax.repeated_calls.contains(&init_call)
        || syntax.repeated_calls.contains(&call)
        || declaration.1 >= call.0
    {
        return None;
    }
    let context = syntax.contexts.get(&declaration)?;
    if result.references.iter().any(|usage| {
        let span = (usage.span.start, usage.span.end);
        !usage.runtime
            || syntax.contexts.get(&span) != Some(context)
            || syntax.assignments.writes.contains_key(&span)
            || syntax.assignments.unsupported_writes.contains(&span)
            || !((span == use_span && span.0 > init_call.1)
                || (syntax.value_reads.get(&span) == Some(&call) && span.0 > init_call.1))
    }) {
        return None;
    }
    let has_text = result
        .references
        .iter()
        .any(|usage| (usage.span.start, usage.span.end) == use_span);
    let has_values = result
        .references
        .iter()
        .any(|usage| syntax.value_reads.get(&(usage.span.start, usage.span.end)) == Some(&call));
    if !has_text || !has_values || result.references.len() != 2 {
        return None;
    }
    bindings.owner(callee)
}

pub(super) fn local_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    call: &super::syntax::Call,
) -> Option<Summary> {
    let owner = candidate(syntax, bindings, call)?;
    (!owner.imported).then(|| source_proof(syntax, bindings, owner))?
}
