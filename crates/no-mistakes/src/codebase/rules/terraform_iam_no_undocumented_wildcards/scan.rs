use hcl::edit::expr::Expression;
use hcl::edit::parser::{self, Error};
use hcl::edit::structure::Block;
use hcl::edit::Span;

pub(super) fn wildcard_resources(source: &str) -> Result<Vec<usize>, Error> {
    let body = parser::parse_body(source)?;
    Ok(body
        .get_blocks("data")
        .filter(|block| {
            matches!(
                block.labels.as_slice(),
                [data_type, _] if data_type.as_str() == "aws_iam_policy_document"
            )
        })
        .flat_map(|block| block.body.get_blocks("statement"))
        .filter_map(statement_wildcard)
        .collect())
}

fn statement_wildcard(statement: &Block) -> Option<usize> {
    if statement.is_labeled()
        || statement
            .body
            .get_attribute("effect")
            .is_some_and(|effect| unparenthesize(&effect.value).as_str() != Some("Allow"))
    {
        return None;
    }

    let resources = statement.body.get_attribute("resources")?;
    let items = unparenthesize(&resources.value).as_array()?;
    if !items
        .iter()
        .filter_map(|item| unparenthesize(item).as_str())
        .any(has_wildcard)
    {
        return None;
    }

    resources.key.span().map(|span| span.start)
}

fn has_wildcard(mut value: &str) -> bool {
    while let Some(index) = value.find(['*', '?', '$', '&']) {
        let tail = &value[index..];
        if tail.starts_with(['*', '?']) {
            return true;
        }
        if let Some(variable) = tail.strip_prefix("${").or_else(|| tail.strip_prefix("&{")) {
            let Some((_, rest)) = variable.split_once('}') else {
                return false;
            };
            value = rest;
        } else {
            value = &tail[1..];
        }
    }
    false
}

fn unparenthesize(mut expression: &Expression) -> &Expression {
    while let Expression::Parenthesis(parenthesis) = expression {
        expression = parenthesis.inner();
    }
    expression
}
