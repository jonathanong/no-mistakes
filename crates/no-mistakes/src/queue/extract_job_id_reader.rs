use super::{message, KIND};
use crate::ast::{js_literal, property_key_name, template_quasi_text};
use crate::fx::FxHashMap;
use crate::queue::extract_helpers::static_string;
use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind,
};
use oxc_syntax::symbol::SymbolId;

#[derive(Clone)]
pub(super) enum JobLabel {
    Literal(String),
    Const(SymbolId),
}

impl JobLabel {
    pub(super) fn read(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> Option<Self> {
        let expr = expr.get_inner_expression();
        if let Expression::Identifier(id) = expr {
            return super::referenced_symbol(visitor, id).map(Self::Const);
        }
        static_string(expr).map(Self::Literal)
    }

    fn resolve(self, consts: &FxHashMap<SymbolId, String>) -> Option<String> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Const(symbol) => consts.get(&symbol).cloned(),
        }
    }
}

enum InvalidId {
    Message(String),
    Const(SymbolId),
}

pub(super) struct Evidence {
    id: InvalidId,
    job: Option<JobLabel>,
    line: usize,
}

impl Evidence {
    pub(super) fn finding(self, consts: &FxHashMap<SymbolId, String>) -> Option<LocalCheck> {
        let message = match self.id {
            InvalidId::Message(message) => message,
            InvalidId::Const(symbol) => message::invalid_message(consts.get(&symbol)?)?,
        };
        Some(LocalCheck {
            line: self.line,
            kind: KIND.to_string(),
            job: self.job.and_then(|label| label.resolve(consts)),
            message,
        })
    }
}

pub(super) struct Reader<'visitor, 'a> {
    visitor: &'visitor QueueVisitor<'a>,
    pub(super) evidence: Vec<Evidence>,
}

impl<'visitor, 'a> Reader<'visitor, 'a> {
    pub(super) fn new(visitor: &'visitor QueueVisitor<'a>) -> Self {
        Self {
            visitor,
            evidence: Vec::new(),
        }
    }

    pub(super) fn options(&mut self, job: Option<JobLabel>, options: &Expression<'_>) {
        if let Expression::ObjectExpression(object) = options.get_inner_expression() {
            self.direct(job, object);
        }
    }

    pub(super) fn bulk(&mut self, arg: Option<&Argument<'_>>) {
        if let Some(Argument::ArrayExpression(array)) = arg {
            for element in &array.elements {
                if let ArrayExpressionElement::ObjectExpression(child) = element {
                    self.node(child, false);
                }
            }
        }
    }

    pub(super) fn node_arg(&mut self, arg: Option<&Argument<'_>>) {
        if let Some(Expression::ObjectExpression(object)) = arg
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression)
        {
            self.node(object, true);
        }
    }

    fn node(&mut self, object: &ObjectExpression<'_>, walk_children: bool) {
        let job = property_expr(object, "name").and_then(|expr| JobLabel::read(self.visitor, expr));
        self.direct(job.clone(), object);
        if let Some(options) = property_expr(object, "opts") {
            self.options(job, options);
        }
        if walk_children {
            if let Some(Expression::ArrayExpression(children)) =
                property_expr(object, "children").map(Expression::get_inner_expression)
            {
                for element in &children.elements {
                    if let ArrayExpressionElement::ObjectExpression(child) = element {
                        self.node(child, true);
                    }
                }
            }
        }
    }

    fn direct(&mut self, job: Option<JobLabel>, object: &ObjectExpression<'_>) {
        if let Some(value) = property_expr(object, "jobId") {
            self.job_id(job, value);
        }
    }

    pub(super) fn job_id(&mut self, job: Option<JobLabel>, expr: &Expression<'_>) {
        match expr.get_inner_expression() {
            Expression::StringLiteral(literal) => {
                self.push(job, literal.span.start, js_literal(literal).as_ref());
            }
            Expression::TemplateLiteral(template) => {
                for quasi in &template.quasis {
                    self.push(
                        job.clone(),
                        quasi.span.start,
                        template_quasi_text(quasi).as_ref(),
                    );
                }
            }
            Expression::Identifier(id) => {
                if let Some(symbol) = super::referenced_symbol(self.visitor, id) {
                    self.evidence.push(Evidence {
                        id: InvalidId::Const(symbol),
                        job,
                        line: line_number(self.visitor.source, id.span.start),
                    });
                }
            }
            _ => {}
        }
    }

    fn push(&mut self, job: Option<JobLabel>, start: u32, text: &str) {
        if let Some(message) = message::invalid_message(text) {
            self.evidence.push(Evidence {
                line: line_number(self.visitor.source, start),
                job,
                id: InvalidId::Message(message),
            });
        }
    }
}

fn property_expr<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a Expression<'a>> {
    for property in object.properties.iter().rev() {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property_key_name(&property.key)?.as_ref() == name {
            return Some(&property.value);
        }
    }
    None
}
