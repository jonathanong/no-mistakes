fn evaluate_template_literal<'a>(
    tpl: &'a TemplateLiteral<'a>,
    defs: &HashMap<std::borrow::Cow<'a, str>, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    let mut values = vec![String::new()];
    for (index, quasi) in tpl.quasis.iter().enumerate() {
        let cooked = quasi
            .value
            .cooked
            .map(|_| crate::ast::template_quasi_text(quasi))
            .unwrap_or(std::borrow::Cow::Borrowed(""));
        for value in &mut values {
            value.push_str(&cooked);
        }
        if let Some(expr) = tpl.expressions.get(index) {
            let expr_values =
                evaluate_route_expression(expr, defs, imported_helpers, env, depth + 1);
            values = concat_candidates(&values, &expr_values);
        }
    }
    values
}
