pub trait Visitor {
    fn visit_import(&mut self, _import: &ImportDeclaration) {}
    fn visit_expression(&mut self, _expr: &Expression) {}
    fn visit_jsx_opening(&mut self, _opening: &JSXOpeningElement) {}
    fn visit_jsx_expression_container(&mut self, _expr: &JSXExpression, _span_start: u32) {}
}
