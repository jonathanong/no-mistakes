use crate::ast;
use oxc_ast::ast::{Argument, CallExpression};

pub fn is_candidate_url(url: &str) -> bool {
    url.starts_with('/') || url.starts_with("http://") || url.starts_with("https://")
}

pub fn callee_matches_navigation_helper(callee: &Option<Vec<String>>, helpers: &[String]) -> bool {
    let Some(parts) = callee else {
        return false;
    };
    let full_name = parts.join(".");
    helpers.iter().any(|helper| {
        helper == &full_name
            || (!helper.contains('.') && parts.last().is_some_and(|part| part == helper))
    })
}

pub fn callee_has_not(callee: &Option<Vec<String>>) -> bool {
    let Some(parts) = callee else {
        return false;
    };
    parts.iter().any(|part| part == "not")
}

pub fn callee_is_member_named(callee: &oxc_ast::ast::Expression<'_>, method: &str) -> bool {
    match callee {
        oxc_ast::ast::Expression::StaticMemberExpression(member) => member.property.name == method,
        _ => false,
    }
}

pub fn callee_is_playwright_wait_for_url(callee: &oxc_ast::ast::Expression<'_>) -> bool {
    let oxc_ast::ast::Expression::StaticMemberExpression(member) = callee else {
        return false;
    };
    if member.property.name != "waitForURL" {
        return false;
    }

    ast::expression_path(&member.object).is_some_and(|path| {
        path.last()
            .is_some_and(|receiver| matches!(receiver.as_str(), "page" | "frame"))
    })
}

pub fn callee_is_page_url_to_match(callee: &oxc_ast::ast::Expression<'_>) -> bool {
    let oxc_ast::ast::Expression::StaticMemberExpression(member) = callee else {
        return false;
    };
    if member.property.name != "toMatch" {
        return false;
    }

    let Some(expect_call) = expect_call_expression(&member.object) else {
        return false;
    };
    let Some(expect_callee) = ast::expression_path(&expect_call.callee) else {
        return false;
    };
    if !matches!(
        expect_callee
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
        ["expect"] | ["expect", "soft"]
    ) {
        return false;
    }

    let Some(Argument::CallExpression(url_call)) = expect_call.arguments.first() else {
        return false;
    };
    ast::expression_path(&url_call.callee).is_some_and(|path| path == ["page", "url"])
}

pub fn expect_call_expression<'a>(
    expression: &'a oxc_ast::ast::Expression<'a>,
) -> Option<&'a CallExpression<'a>> {
    match expression {
        oxc_ast::ast::Expression::CallExpression(call) => Some(call),
        oxc_ast::ast::Expression::StaticMemberExpression(member)
            if member.property.name == "not" =>
        {
            expect_call_expression(&member.object)
        }
        oxc_ast::ast::Expression::ParenthesizedExpression(parenthesized) => {
            expect_call_expression(&parenthesized.expression)
        }
        _ => None,
    }
}
