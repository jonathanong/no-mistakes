pub fn program_contains_jsx(program: &Program) -> bool {
    struct Probe(bool);
    impl Visitor for Probe {
        fn visit_expression(&mut self, expr: &Expression) {
            if matches!(expr, Expression::JSXElement(_) | Expression::JSXFragment(_)) {
                self.0 = true;
            }
        }
        fn visit_jsx_opening(&mut self, _: &JSXOpeningElement) {
            self.0 = true;
        }
    }
    let mut p = Probe(false);
    walk_program(program, &mut p);
    p.0
}

/// Returns the tag name of a JSXOpeningElement if it is a simple Identifier
/// (e.g. `<a>`, `<Link>`, `<script>`). Returns `None` for namespaced or member
/// expressions (`<foo.bar>`, `<ns:foo>`).
pub fn jsx_identifier_name<'a>(opening: &'a JSXOpeningElement) -> Option<&'a str> {
    match &opening.name {
        oxc::ast::ast::JSXElementName::Identifier(id) => Some(id.name.as_str()),
        oxc::ast::ast::JSXElementName::IdentifierReference(id) => Some(id.name.as_str()),
        _ => None,
    }
}

/// Reads a JSX attribute by name. Returns `(present, string_value_if_string_literal)`.
/// `string_value_if_string_literal` is `Some` only if the value is a string
/// literal (`x="foo"`) or an expression container wrapping a string literal
/// (`x={"foo"}`). Boolean shorthand (`<Foo bar />`) → `(true, None)`.
pub fn find_string_attr<'a>(
    opening: &'a JSXOpeningElement,
    name: &str,
) -> Option<(bool, Option<&'a str>)> {
    for item in &opening.attributes {
        let JSXAttributeItem::Attribute(attr) = item else {
            continue;
        };
        let attr_name = match &attr.name {
            oxc::ast::ast::JSXAttributeName::Identifier(id) => id.name.as_str(),
            _ => continue,
        };
        if attr_name != name {
            continue;
        }
        return Some(jsx_attr_value(&attr.value));
    }
    None
}

fn jsx_attr_value<'a>(value: &'a Option<JSXAttributeValue<'a>>) -> (bool, Option<&'a str>) {
    match value {
        None => (true, None),
        Some(JSXAttributeValue::StringLiteral(s)) => (true, Some(s.value.as_str())),
        Some(JSXAttributeValue::ExpressionContainer(c)) => match c.expression.as_expression() {
            Some(expr) => match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
                Expression::StringLiteral(s) => (true, Some(s.value.as_str())),
                _ => (true, None),
            },
            None => (true, None),
        },
        Some(_) => (true, None),
    }
}

