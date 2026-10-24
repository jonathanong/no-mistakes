use super::*;

pub(super) fn input(
    expression: &Expression<'_>,
    env: &mut FxHashMap<String, Span>,
    tags: &template_binds::TemplateBinds,
) -> Input {
    match unwrap_ts_wrappers(expression) {
        Expression::TaggedTemplateExpression(value) => tags.input(value),
        Expression::StringLiteral(value) => Input::Sql {
            text: value.value.to_string(),
            source_span: span(value.span),
            holes: Vec::new(),
            parameterized: None,
            positions: Vec::new(),
        },
        Expression::TemplateLiteral(value) => {
            let mut text = String::new();
            let mut holes = Vec::new();
            let mut prefix = "no_mistakes_low_hole_".to_string();
            while value.quasis.iter().any(|part| {
                part.value
                    .cooked
                    .as_ref()
                    .is_some_and(|text| text.contains(&prefix))
            }) {
                prefix.push('_');
            }
            for (index, quasi) in value.quasis.iter().enumerate() {
                let Some(cooked) = &quasi.value.cooked else {
                    return Input::Unknown;
                };
                text.push_str(cooked);
                if let Some(expression) = value.expressions.get(index) {
                    // Placeholder identity preserves which SQL position owns each input.
                    let placeholder = format!("{prefix}{index}");
                    text.push_str(&placeholder);
                    let source = chain(expression).and_then(|(root, parts)| {
                        (parts.len() == 3 && parts[0] == "process" && parts[1] == "env").then(
                            || {
                                let key = format!("process.env.{}", parts[2]);
                                env.insert(format!("{key}@{}", root.0), root);
                                format!("{key}@{}", root.0)
                            },
                        )
                    });
                    holes.push((placeholder, source));
                }
            }
            Input::Sql {
                text,
                holes,
                source_span: span(value.span),
                parameterized: None,
                positions: Vec::new(),
            }
        }
        Expression::Identifier(id) => Input::Reference(span(id.span)),
        _ => Input::Unknown,
    }
}

// Indirect receiver operations retain uncertainty without selecting data methods.
pub(super) fn indirect_executor_call(parts: &[String]) -> bool {
    parts.last().is_some_and(|name| {
        (parts.len() > 2 && name == "query")
            || (parts.len() >= 2 && matches!(name.as_str(), "call" | "apply" | "bind"))
    })
}
