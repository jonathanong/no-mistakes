fn walk_expression(expr: &Expression, v: &mut dyn Visitor) {
    v.visit_expression(expr);
    match expr {
        Expression::CallExpression(call) => {
            walk_expression(&call.callee, v);
            walk_args(&call.arguments, v);
        }
        Expression::NewExpression(n) => {
            walk_expression(&n.callee, v);
            walk_args(&n.arguments, v);
        }
        Expression::ChainExpression(chain) => walk_chain_expression(chain, v),
        Expression::AwaitExpression(a) => walk_expression(&a.argument, v),
        Expression::YieldExpression(y) => walk_optional_expression(y.argument.as_ref(), v),
        Expression::StaticMemberExpression(m) => walk_expression(&m.object, v),
        Expression::ComputedMemberExpression(m) => {
            walk_expression(&m.object, v);
            walk_expression(&m.expression, v);
        }
        Expression::AssignmentExpression(a) => {
            walk_member_expression(a.left.as_member_expression(), v);
            walk_expression(&a.right, v);
        }
        Expression::ArrowFunctionExpression(a) => walk_statements(&a.body.statements, v),
        Expression::FunctionExpression(f) => {
            walk_function_body(f.body.as_deref(), v);
        }
        Expression::ConditionalExpression(c) => {
            walk_expression(&c.test, v);
            walk_expression(&c.consequent, v);
            walk_expression(&c.alternate, v);
        }
        Expression::LogicalExpression(l) => {
            walk_expression(&l.left, v);
            walk_expression(&l.right, v);
        }
        Expression::BinaryExpression(b) => {
            walk_expression(&b.left, v);
            walk_expression(&b.right, v);
        }
        Expression::UnaryExpression(u) => walk_expression(&u.argument, v),
        Expression::UpdateExpression(u) => {
            walk_member_expression(u.argument.as_member_expression(), v);
        }
        Expression::SequenceExpression(s) => walk_expressions(&s.expressions, v),
        Expression::ObjectExpression(o) => walk_object_props(&o.properties, v),
        Expression::ArrayExpression(a) => walk_array_elems(&a.elements, v),
        Expression::ParenthesizedExpression(p) => walk_expression(&p.expression, v),
        Expression::TSAsExpression(t) => walk_expression(&t.expression, v),
        Expression::TSNonNullExpression(t) => walk_expression(&t.expression, v),
        Expression::TSSatisfiesExpression(t) => walk_expression(&t.expression, v),
        Expression::TSTypeAssertion(t) => walk_expression(&t.expression, v),
        Expression::TaggedTemplateExpression(t) => walk_expression(&t.tag, v),
        Expression::TemplateLiteral(t) => walk_expressions(&t.expressions, v),
        Expression::JSXElement(elem) => walk_jsx_element(elem, v),
        Expression::JSXFragment(frag) => walk_jsx_children(&frag.children, v),
        _ => {}
    }
}

fn walk_args(args: &[Argument], v: &mut dyn Visitor) {
    for arg in args {
        walk_argument(arg, v);
    }
}

fn walk_chain_expression(chain: &ChainExpression, v: &mut dyn Visitor) {
    match &chain.expression {
        oxc_ast::ast::ChainElement::CallExpression(call) => {
            walk_expression(&call.callee, v);
            walk_args(&call.arguments, v);
        }
        other => {
            walk_member_expression(other.as_member_expression(), v);
        }
    }
}

fn walk_expressions(exprs: &[Expression], v: &mut dyn Visitor) {
    for expr in exprs {
        walk_expression(expr, v);
    }
}

fn walk_object_props(props: &[ObjectPropertyKind], v: &mut dyn Visitor) {
    for prop in props {
        match prop {
            ObjectPropertyKind::ObjectProperty(p) => walk_expression(&p.value, v),
            ObjectPropertyKind::SpreadProperty(s) => walk_expression(&s.argument, v),
        }
    }
}

fn walk_array_elems(elems: &[ArrayExpressionElement], v: &mut dyn Visitor) {
    for elem in elems {
        if let ArrayExpressionElement::SpreadElement(s) = elem {
            walk_expression(&s.argument, v);
        } else if let Some(e) = elem.as_expression() {
            walk_expression(e, v);
        }
    }
}

fn walk_jsx_children(children: &[JSXChild], v: &mut dyn Visitor) {
    for child in children {
        walk_jsx_child(child, v);
    }
}

fn walk_member_expression(
    member: Option<&oxc_ast::ast::MemberExpression<'_>>,
    v: &mut dyn Visitor,
) {
    if let Some(member) = member {
        walk_expression(member.object(), v);
        if let oxc_ast::ast::MemberExpression::ComputedMemberExpression(cm) = member {
            walk_expression(&cm.expression, v);
        }
    }
}

fn walk_argument(arg: &Argument, v: &mut dyn Visitor) {
    match arg {
        Argument::SpreadElement(s) => walk_expression(&s.argument, v),
        _ => {
            walk_optional_expression(arg.as_expression(), v);
        }
    }
}

fn walk_optional_expression(expr: Option<&Expression>, v: &mut dyn Visitor) {
    let _ = expr.map(|expr| walk_expression(expr, v));
}

fn walk_optional_statement(stmt: Option<&Statement>, v: &mut dyn Visitor) {
    let _ = stmt.map(|stmt| walk_statement(stmt, v));
}

fn walk_optional_declaration(decl: Option<&Declaration>, v: &mut dyn Visitor) {
    let _ = decl.map(|decl| walk_declaration(decl, v));
}

fn walk_jsx_element(elem: &JSXElement, v: &mut dyn Visitor) {
    v.visit_jsx_opening(&elem.opening_element);
    for attr in &elem.opening_element.attributes {
        match attr {
            JSXAttributeItem::Attribute(a) => {
                if let Some(JSXAttributeValue::ExpressionContainer(c)) = &a.value {
                    v.visit_jsx_expression_container(&c.expression, c.span.start);
                    if let Some(expr) = c.expression.as_expression() {
                        walk_expression(expr, v);
                    }
                }
            }
            JSXAttributeItem::SpreadAttribute(s) => {
                walk_expression(&s.argument, v);
            }
        }
    }
    walk_jsx_children(&elem.children, v);
}

fn walk_jsx_child(child: &JSXChild, v: &mut dyn Visitor) {
    match child {
        JSXChild::Element(elem) => walk_jsx_element(elem, v),
        JSXChild::Fragment(frag) => walk_jsx_children(&frag.children, v),
        JSXChild::ExpressionContainer(container) => {
            v.visit_jsx_expression_container(&container.expression, container.span.start);
            if let Some(expr) = container.expression.as_expression() {
                walk_expression(expr, v);
            }
        }
        JSXChild::Spread(s) => walk_expression(&s.expression, v),
        _ => {}
    }
}

