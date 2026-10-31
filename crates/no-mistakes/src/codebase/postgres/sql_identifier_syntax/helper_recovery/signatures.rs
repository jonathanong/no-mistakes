use super::{lower, span, CallableId, FileModels, FunctionModel};
use oxc_ast::ast::{BindingPattern, FormalParameters, Function};

impl FileModels {
    pub fn arrow(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'_>,
        ctx: &lower::Context<'_>,
    ) {
        self.signature(CallableId(function.span.start), None, &function.params, ctx);
        if let Some(lowered) = lower::arrow(function, ctx) {
            let id = CallableId(function.span.start);
            self.functions.insert(
                id,
                FunctionModel {
                    id,
                    span: span(function.span),
                    binding: None,
                    lowered,
                },
            );
        }
    }
    pub fn function(&mut self, function: &Function<'_>, ctx: &lower::Context<'_>) {
        if function.body.is_some() {
            self.signature(
                CallableId(function.span.start),
                function.id.as_ref().and_then(|id| ctx.bindings.id(id.span)),
                &function.params,
                ctx,
            );
        }
        let Some(lowered) = lower::function(function, ctx) else {
            return;
        };
        let id = CallableId(function.span.start);
        self.functions.insert(
            id,
            FunctionModel {
                id,
                span: span(function.span),
                binding: function.id.as_ref().and_then(|id| ctx.bindings.id(id.span)),
                lowered,
            },
        );
    }
    fn signature(
        &mut self,
        callable: CallableId,
        binding: Option<usize>,
        parameters: &FormalParameters<'_>,
        ctx: &lower::Context<'_>,
    ) {
        for parameter in &parameters.items {
            let (BindingPattern::BindingIdentifier(binding), Some(annotation)) =
                (&parameter.pattern, &parameter.type_annotation)
            else {
                continue;
            };
            let oxc_ast::ast::TSType::TSFunctionType(callback) = &annotation.type_annotation else {
                continue;
            };
            if let Some(binding) = ctx.bindings.id(binding.span) {
                self.callbacks
                    .insert(binding, parameter_bindings(&callback.params, ctx));
            }
        }
        self.signatures
            .insert(callable, parameter_bindings(parameters, ctx));
        if let Some(binding) = binding {
            self.signature_binding(binding, callable);
        }
    }
    pub(super) fn signature_binding(&mut self, binding: usize, callable: CallableId) {
        self.signature_functions
            .entry(binding)
            .and_modify(|id| *id = None)
            .or_insert(Some(callable));
    }
}

fn parameter_bindings(
    parameters: &FormalParameters<'_>,
    ctx: &lower::Context<'_>,
) -> Vec<Option<usize>> {
    parameters
        .items
        .iter()
        .map(|parameter| {
            if let BindingPattern::BindingIdentifier(id) = &parameter.pattern {
                ctx.bindings.id(id.span)
            } else {
                None
            }
        })
        .collect()
}
