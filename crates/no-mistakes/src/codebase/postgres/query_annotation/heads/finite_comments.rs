//! Finite literal names in the first ordinary SQL comment only.
use super::Span;
use crate::fx::FxHashSet;
mod domain;
mod grammar;
mod proof;
pub(super) mod scalar;
mod scalar_domain;
mod scalar_grammar;
mod scalar_ownership;
pub(super) use grammar::{loop_domain, root, template};
pub(super) use proof::proofs;

#[derive(Clone)]
pub(super) struct Domain {
    pub declaration: Span,
    pub name: String,
    pub values: Vec<String>,
}
pub(super) struct TemplateUse {
    pub call: Span,
    pub reference: Span,
    pub domain: Domain,
    pub before: String,
    pub after: String,
}
pub(super) struct Root {
    pub declaration: Span,
    pub function: Span,
    pub reference: Span,
    pub tag: Span,
    pub before: String,
    pub after: String,
    pub scalar: Option<Span>,
}
pub(super) struct Function {
    pub declaration: Span,
    pub body: Span,
    pub parameter: Span,
    pub count: usize,
    pub scalar_plain: bool,
}
pub(super) struct Select {
    pub declaration: Span,
    pub body: Span,
    pub enclosing: Span,
    pub parameter: Span,
}
pub(super) enum Argument {
    Literal(String),
    Reference(Span),
    Other,
}
pub(super) struct CallSite {
    pub callee: Span,
    pub first: Argument,
    pub count: usize,
    pub valid: bool,
    pub context: Option<Span>,
}
pub(super) fn declarations(syntax: &super::syntax::Syntax) -> FxHashSet<Span> {
    syntax
        .finite_functions
        .iter()
        .flat_map(|value| [value.declaration, value.parameter])
        .chain(
            syntax
                .finite_selects
                .iter()
                .flat_map(|value| [value.declaration, value.parameter]),
        )
        .chain(
            syntax
                .finite_templates
                .iter()
                .map(|value| value.domain.declaration),
        )
        .collect()
}
