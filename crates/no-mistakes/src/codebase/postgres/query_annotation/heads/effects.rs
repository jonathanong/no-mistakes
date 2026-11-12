//! Prefix-preserving effects of a discarded call on a trusted local builder.
use super::Span;
use oxc_ast::ast::{BindingPattern, Function, FunctionType};

mod calls;
pub(super) use calls::call;
mod candidates;
mod conditional;
mod defaults;
mod prepare;
mod resolve;
mod summary;
pub(super) use conditional::collect as collect_conditionals;
pub(crate) use prepare::Pending;
pub(super) use prepare::{prepare, Prepared};
pub(super) use resolve::Resolver;

#[derive(Clone, Debug)]
pub(crate) struct Import {
    pub specifier: String,
    pub imported: String,
    pub shape: CallShape,
}

#[derive(Clone, Debug)]
pub(crate) struct Summary {
    pub signature: Signature,
    pub required: Vec<Import>,
}

#[derive(Clone, Debug)]
pub(crate) struct Conditional {
    pub spans: Vec<Span>,
    pub required: Vec<Span>,
}

#[derive(Clone, Copy)]
pub(super) struct FunctionSyntax {
    pub parameter: Span,
    pub position: u8,
    pub function: Span,
    pub body: Span,
    pub signature: Signature,
    pub returned: Option<Span>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Signature {
    required_objects: u16,
    empty_defaults: u16,
    returns_builder: bool,
}

impl Signature {
    pub(crate) fn accepts(self, shape: CallShape) -> bool {
        (!shape.returned_append || self.returns_builder)
            && (0..16)
                .filter(|index| self.required_objects & (1 << index) != 0)
                .all(|index| {
                    if index < shape.arg_count {
                        shape.closed_objects & (1 << index) != 0
                    } else {
                        self.empty_defaults & (1 << index) != 0
                    }
                })
    }

    pub(super) fn plain(self) -> bool {
        self.required_objects == 0
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CallShape {
    arg_count: usize,
    closed_objects: u16,
    returned_append: bool,
}

#[derive(Clone, Copy)]
pub(super) struct CallSyntax {
    pub span: Span,
    pub callee: Span,
    pub builder: Span,
    pub position: u8,
    pub shape: CallShape,
}

pub(super) fn function(value: &Function<'_>) -> [Option<FunctionSyntax>; 2] {
    if value.r#type != FunctionType::FunctionDeclaration || value.r#async || value.generator {
        return [None, None];
    }
    let Some(body) = value.body.as_ref() else {
        return [None, None];
    };
    let params = &value.params;
    if params.rest.is_some() || params.items.is_empty() {
        return [None, None];
    }
    let mut signature = Signature::default();
    for (index, param) in params.items.iter().enumerate() {
        if !param.decorators.is_empty()
            || !defaults::plain_parameter(
                index,
                &param.pattern,
                param.initializer.as_deref(),
                &mut signature,
            )
        {
            return [None, None];
        }
    }
    std::array::from_fn(|position| {
        // Keep the existing first-parameter grammar; the new position has no defaults.
        if position == 1
            && !(params.items.len() == 2
                && params.items.iter().all(|param| {
                    param.initializer.is_none()
                        && matches!(param.pattern, BindingPattern::BindingIdentifier(_))
                }))
        {
            return None;
        }
        let BindingPattern::BindingIdentifier(parameter) = &params.items.get(position)?.pattern
        else {
            return None;
        };
        Some(FunctionSyntax {
            parameter: (parameter.span.start, parameter.span.end),
            position: position as u8,
            function: (value.span.start, value.span.end),
            body: (body.span.start, body.span.end),
            signature,
            returned: body.statements.last().and_then(|statement| {
                let oxc_ast::ast::Statement::ReturnStatement(value) = statement else {
                    return None;
                };
                value
                    .argument
                    .as_ref()
                    .and_then(super::append::prefix_reference)
            }),
        })
    })
}
