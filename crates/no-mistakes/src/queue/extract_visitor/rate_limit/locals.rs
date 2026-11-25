use super::QueueVisitor;
use crate::ast::property_key_name;
use crate::queue::extract_job_id::referenced_symbol;
use oxc_ast::ast::{
    BindingPattern, Class, Expression, ObjectExpression, ObjectPropertyKind, VariableDeclarator,
};
use oxc_syntax::symbol::SymbolId;

#[derive(Clone, Copy)]
pub(super) enum ObjProp {
    Absent,
    NonQueue,
    Symbol(Option<SymbolId>),
    Opaque,
}

pub(super) fn callable_start(expr: &Expression<'_>) -> Option<u32> {
    match expr.get_inner_expression() {
        Expression::FunctionExpression(function) => Some(function.span.start),
        Expression::ArrowFunctionExpression(arrow) => Some(arrow.span.start),
        _ => None,
    }
}

impl QueueVisitor<'_> {
    pub(in crate::queue) fn note_rate_limit_class(&mut self, class: &Class<'_>) {
        if !self.shared_rate_limit.enabled {
            return;
        }
        if let Some(symbol) = class
            .id
            .as_ref()
            .and_then(|id| self.stable_rate_limit_symbol(id))
            .filter(|_| self.is_non_queue_class(class))
        {
            self.shared_rate_limit.local_non_queue.insert(symbol);
        }
    }

    pub(super) fn note_rate_limit_local_const(&mut self, declaration: &VariableDeclarator<'_>) {
        let (BindingPattern::BindingIdentifier(id), Some(init)) = (
            &declaration.id,
            declaration
                .init
                .as_ref()
                .map(Expression::get_inner_expression),
        ) else {
            return;
        };
        let Some(symbol) = self.stable_rate_limit_symbol(id) else {
            return;
        };
        match init {
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
                self.shared_rate_limit.local_non_queue.insert(symbol);
            }
            Expression::ClassExpression(class) if self.is_non_queue_class(class) => {
                self.shared_rate_limit.local_non_queue.insert(symbol);
            }
            Expression::Identifier(rhs) => {
                if let Some(rhs_sym) = referenced_symbol(self, rhs) {
                    self.shared_rate_limit.local_aliases.insert(symbol, rhs_sym);
                } else if rhs.name != "RateLimitError" {
                    self.shared_rate_limit.local_non_queue.insert(symbol);
                }
            }
            Expression::ObjectExpression(obj) => {
                let prop = self.classify_object_rate_limit_prop(obj);
                self.shared_rate_limit
                    .local_object_rate_limit
                    .insert(symbol, prop);
            }
            _ => {}
        }
    }

    fn is_non_queue_class(&self, class: &Class<'_>) -> bool {
        let Some(heritage) = &class.heritage else {
            return true;
        };
        let Expression::Identifier(id) = heritage.expression.get_inner_expression() else {
            return false;
        };
        match referenced_symbol(self, id) {
            None => id.name != "RateLimitError",
            Some(sym) => self.shared_rate_limit.local_non_queue.contains(&sym),
        }
    }

    fn classify_object_rate_limit_prop(&self, obj: &ObjectExpression<'_>) -> ObjProp {
        let mut state = ObjProp::Absent;
        for prop in &obj.properties {
            match prop {
                ObjectPropertyKind::ObjectProperty(prop) => {
                    let Some(key) = property_key_name(&prop.key).filter(|_| !prop.computed) else {
                        state = ObjProp::Opaque;
                        continue;
                    };
                    if key == "RateLimitError" {
                        state = match prop.value.get_inner_expression() {
                            Expression::Identifier(id) => {
                                ObjProp::Symbol(referenced_symbol(self, id))
                            }
                            Expression::FunctionExpression(_)
                            | Expression::ArrowFunctionExpression(_) => ObjProp::NonQueue,
                            Expression::ClassExpression(class)
                                if self.is_non_queue_class(class) =>
                            {
                                ObjProp::NonQueue
                            }
                            _ => ObjProp::Opaque,
                        };
                    }
                }
                ObjectPropertyKind::SpreadProperty(_) => state = ObjProp::Opaque,
            }
        }
        state
    }
}
