use crate::playwright::ast;

pub(super) fn direct_child_texts(
    children: &[oxc_ast::ast::JSXChild<'_>],
    source: &str,
) -> Vec<String> {
    child_texts(children, source, false)
}

pub(super) fn descendant_texts(
    children: &[oxc_ast::ast::JSXChild<'_>],
    source: &str,
) -> Vec<String> {
    child_texts(children, source, true)
}

fn child_texts(
    children: &[oxc_ast::ast::JSXChild<'_>],
    source: &str,
    include_descendants: bool,
) -> Vec<String> {
    let mut results = Vec::new();
    let mut current = String::new();

    for child in children {
        match child {
            oxc_ast::ast::JSXChild::Text(text) => {
                current.push_str(text.value.as_str());
            }
            oxc_ast::ast::JSXChild::ExpressionContainer(container) => match &container.expression {
                oxc_ast::ast::JSXExpression::StringLiteral(literal) => {
                    current.push_str(literal.value.as_str());
                }
                oxc_ast::ast::JSXExpression::NumericLiteral(literal) => {
                    current.push_str(&literal.value.to_string());
                }
                oxc_ast::ast::JSXExpression::TemplateLiteral(template)
                    if template.expressions.is_empty() =>
                {
                    current.push_str(&ast::template_literal_text(template, source));
                }
                _ if !current.is_empty() => {
                    results.push(std::mem::take(&mut current));
                }
                _ => {}
            },
            oxc_ast::ast::JSXChild::Element(element) if include_descendants => {
                append_descendant_segments(
                    &mut results,
                    &mut current,
                    descendant_texts(&element.children, source),
                );
            }
            oxc_ast::ast::JSXChild::Fragment(fragment) if include_descendants => {
                append_descendant_segments(
                    &mut results,
                    &mut current,
                    descendant_texts(&fragment.children, source),
                );
            }
            _ => {
                if !current.is_empty() {
                    results.push(std::mem::take(&mut current));
                }
            }
        }
    }

    if !current.is_empty() {
        results.push(current);
    }

    results
}

fn append_descendant_segments(
    results: &mut Vec<String>,
    current: &mut String,
    segments: Vec<String>,
) {
    for (index, segment) in segments.into_iter().enumerate() {
        if index > 0 && !current.is_empty() {
            results.push(std::mem::take(current));
        }
        current.push_str(&segment);
    }
}
