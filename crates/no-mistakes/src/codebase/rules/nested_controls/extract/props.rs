use oxc_ast::ast::{
    Expression, JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild,
    JSXElementName, JSXExpression, JSXOpeningElement,
};

pub(crate) struct Location {
    pub(crate) line: usize,
    pub(crate) kind: Kind,
}

pub(crate) enum Kind {
    Nested {
        inner: &'static str,
        outer: &'static str,
    },
    Unanalyzable(UnanalyzableReason),
}

pub(crate) enum UnanalyzableReason {
    UncertainHref,
    ChildExpression,
    ChildSpread,
    WrapperTag(Option<String>),
}

impl UnanalyzableReason {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::UncertainHref => "<a> has a non-literal or spread-overwritten href, so control nesting cannot be proven. Use a literal href string, remove the trailing spread, or suppress with a reason.".into(),
            Self::ChildExpression => "JSX child expression inside a button or link cannot be proven free of nested controls. Render static sibling elements or suppress with a reason.".into(),
            Self::ChildSpread => "JSX spread inside a button or link cannot be proven free of nested controls. Render static sibling elements or suppress with a reason.".into(),
            Self::WrapperTag(Some(name)) => format!("<{name}> inside a button or link is outside the supported static wrapper tags (a, button, div, em, span, strong), so nested controls cannot be proven absent. Use a supported native wrapper, move controls to siblings, or suppress with a reason."),
            Self::WrapperTag(None) => "Unresolved JSX element inside a button or link cannot be proven free of nested controls. Use a supported native wrapper, move controls to siblings, or suppress with a reason.".into(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Ancestor {
    Definite(&'static str),
    UncertainLink,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HrefStatus {
    Literal,
    Absent,
    Uncertain,
}

pub(super) enum ChildrenProp<'a> {
    Absent,
    Spread,
    Attribute(usize, &'a JSXAttribute<'a>),
}

pub(super) fn is_allowed_wrapper(tag: Option<&str>) -> bool {
    matches!(tag, Some("a" | "button" | "div" | "em" | "span" | "strong"))
}

pub(super) fn tag_name<'a>(opening: &'a JSXOpeningElement<'_>) -> Option<&'a str> {
    match &opening.name {
        JSXElementName::Identifier(name) => Some(name.name.as_str()),
        JSXElementName::IdentifierReference(name) => Some(name.name.as_str()),
        _ => None,
    }
}

pub(super) fn classify_control(
    tag: Option<&str>,
    opening: &JSXOpeningElement<'_>,
) -> Option<Ancestor> {
    match tag {
        Some("button") => Some(Ancestor::Definite("button")),
        Some("a") => match href_status(opening) {
            HrefStatus::Literal => Some(Ancestor::Definite("a")),
            HrefStatus::Uncertain => Some(Ancestor::UncertainLink),
            HrefStatus::Absent => None,
        },
        _ => None,
    }
}

pub(super) fn next_ancestor(
    control: Option<Ancestor>,
    ancestor: Option<Ancestor>,
) -> Option<Ancestor> {
    match (control, ancestor) {
        (Some(def @ Ancestor::Definite(_)), _) | (_, Some(def @ Ancestor::Definite(_))) => {
            Some(def)
        }
        (Some(uncertain), _) | (_, Some(uncertain)) => Some(uncertain),
        (None, None) => None,
    }
}

fn href_status(opening: &JSXOpeningElement<'_>) -> HrefStatus {
    for item in opening.attributes.iter().rev() {
        match item {
            JSXAttributeItem::SpreadAttribute(_) => return HrefStatus::Uncertain,
            JSXAttributeItem::Attribute(attribute) if matches!(&attribute.name, JSXAttributeName::Identifier(name) if name.name == "href") =>
            {
                return match &attribute.value {
                    Some(JSXAttributeValue::StringLiteral(_)) => HrefStatus::Literal,
                    Some(JSXAttributeValue::ExpressionContainer(value)) => {
                        match &value.expression {
                            JSXExpression::StringLiteral(_) => HrefStatus::Literal,
                            JSXExpression::NullLiteral(_) => HrefStatus::Absent,
                            _ => HrefStatus::Uncertain,
                        }
                    }
                    _ => HrefStatus::Uncertain,
                };
            }
            _ => {}
        }
    }
    HrefStatus::Absent
}

pub(super) fn has_effective_jsx_children(children: &[JSXChild<'_>]) -> bool {
    children.iter().any(|child| match child {
        JSXChild::Element(_) | JSXChild::Fragment(_) | JSXChild::Spread(_) => true,
        JSXChild::ExpressionContainer(container) => {
            !matches!(container.expression, JSXExpression::EmptyExpression(_))
        }
        JSXChild::Text(text) => !is_ignored_jsx_whitespace(&text.value),
    })
}

fn is_ignored_jsx_whitespace(text: &str) -> bool {
    text.is_empty() || (text.contains(['\n', '\r']) && text.trim().is_empty())
}

pub(super) fn children_prop<'a>(opening: &'a JSXOpeningElement<'a>) -> ChildrenProp<'a> {
    for (index, item) in opening.attributes.iter().enumerate().rev() {
        match item {
            JSXAttributeItem::SpreadAttribute(_) => return ChildrenProp::Spread,
            JSXAttributeItem::Attribute(attribute) if matches!(&attribute.name, JSXAttributeName::Identifier(name) if name.name == "children") =>
            {
                return ChildrenProp::Attribute(index, attribute);
            }
            _ => {}
        }
    }
    ChildrenProp::Absent
}

pub(super) fn is_safe_literal_child(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
    )
}
