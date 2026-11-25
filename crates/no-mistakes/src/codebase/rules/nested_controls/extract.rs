use oxc_ast::ast::{
    JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement, JSXElementName,
    JSXExpression, JSXFragment, JSXOpeningElement, Program,
};
use oxc_ast_visit::Visit;

pub(crate) struct Location {
    pub(super) line: usize,
    pub(super) kind: Kind,
}

pub(crate) enum Kind {
    Nested { inner: &'static str, outer: &'static str },
    Unanalyzable(UnanalyzableReason),
}

pub(crate) enum UnanalyzableReason {
    UncertainHref,
    ChildExpression,
    ChildSpread,
    WrapperTag(Option<String>),
}

#[derive(Clone, Copy)]
enum Ancestor {
    Definite(&'static str),
    UncertainLink,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HrefStatus {
    Literal,
    Absent,
    Uncertain,
}

#[derive(Default)]
struct Collector {
    ancestor: Option<Ancestor>,
    matches: Vec<(u32, Kind)>,
}

pub(super) fn collect(source: &str, program: &Program<'_>) -> Vec<Location> {
    let mut collector = Collector::default();
    collector.visit_program(program);
    if collector.matches.is_empty() {
        return Vec::new();
    }
    let newlines = memchr::memchr_iter(b'\n', source.as_bytes()).collect::<Vec<_>>();
    collector
        .matches
        .into_iter()
        .map(|(offset, kind)| Location {
            line: newlines.partition_point(|newline| *newline < offset as usize) + 1,
            kind,
        })
        .collect()
}

fn tag_name<'a>(opening: &'a JSXOpeningElement<'_>) -> Option<&'a str> {
    match &opening.name {
        JSXElementName::Identifier(name) => Some(name.name.as_str()),
        JSXElementName::IdentifierReference(name) => Some(name.name.as_str()),
        _ => None,
    }
}

fn has_spread_attribute(opening: &JSXOpeningElement<'_>) -> bool {
    opening.attributes.iter().any(|item| matches!(item, JSXAttributeItem::SpreadAttribute(_)))
}

fn href_status(opening: &JSXOpeningElement<'_>) -> HrefStatus {
    for item in opening.attributes.iter().rev() {
        match item {
            JSXAttributeItem::SpreadAttribute(_) => return HrefStatus::Uncertain,
            JSXAttributeItem::Attribute(attribute)
                if matches!(&attribute.name, JSXAttributeName::Identifier(name) if name.name == "href") =>
            {
                return match &attribute.value {
                    Some(JSXAttributeValue::StringLiteral(_)) => HrefStatus::Literal,
                    Some(JSXAttributeValue::ExpressionContainer(value)) => match &value.expression {
                        JSXExpression::StringLiteral(_) => HrefStatus::Literal,
                        JSXExpression::NullLiteral(_) => HrefStatus::Absent,
                        _ => HrefStatus::Uncertain,
                    },
                    _ => HrefStatus::Uncertain,
                };
            }
            _ => {}
        }
    }
    HrefStatus::Absent
}

fn next_ancestor(control: Option<Ancestor>, ancestor: Option<Ancestor>) -> Option<Ancestor> {
    match (control, ancestor) {
        (Some(def @ Ancestor::Definite(_)), _) | (_, Some(def @ Ancestor::Definite(_))) => Some(def),
        (Some(uncertain), _) | (_, Some(uncertain)) => Some(uncertain),
        (None, None) => None,
    }
}

impl Collector {
    fn children<'a>(&mut self, children: &[JSXChild<'a>], ancestor: Option<Ancestor>) {
        for child in children {
            match child {
                JSXChild::Element(_) | JSXChild::Fragment(_) => self.ancestor = ancestor,
                JSXChild::Text(_) => self.ancestor = None,
                JSXChild::ExpressionContainer(container) => {
                    if ancestor.is_some()
                        && !matches!(container.expression, JSXExpression::EmptyExpression(_))
                    {
                        self.matches.push((
                            container.span.start,
                            Kind::Unanalyzable(UnanalyzableReason::ChildExpression),
                        ));
                    }
                    self.ancestor = None;
                }
                JSXChild::Spread(spread) => {
                    if ancestor.is_some() {
                        self.matches.push((
                            spread.span.start,
                            Kind::Unanalyzable(UnanalyzableReason::ChildSpread),
                        ));
                    }
                    self.ancestor = None;
                }
            }
            self.visit_jsx_child(child);
        }
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        let opening = &element.opening_element;
        let ancestor = self.ancestor.take();
        let tag = tag_name(opening);
        // SVG and MathML use a different native element namespace.
        if matches!(tag, Some("svg" | "math")) {
            self.ancestor = ancestor;
            return;
        }
        let control = match tag {
            Some("button") => Some(Ancestor::Definite("button")),
            Some("a") => match href_status(opening) {
                HrefStatus::Literal => Some(Ancestor::Definite("a")),
                HrefStatus::Uncertain => Some(Ancestor::UncertainLink),
                HrefStatus::Absent => None,
            },
            _ => None,
        };
        if let Some(outer) = ancestor {
            match (control, outer) {
                (Some(Ancestor::Definite(inner)), Ancestor::Definite(outer)) => {
                    self.matches.push((opening.span.start, Kind::Nested { inner, outer }));
                }
                (Some(_), _) => {
                    self.matches.push((
                        opening.span.start,
                        Kind::Unanalyzable(UnanalyzableReason::UncertainHref),
                    ));
                }
                (None, _) if !matches!(tag, Some("a" | "button" | "div" | "em" | "span" | "strong")) => {
                    self.matches.push((
                        opening.span.start,
                        Kind::Unanalyzable(UnanalyzableReason::WrapperTag(tag.map(str::to_owned))),
                    ));
                }
                (None, _) => {}
            }
        }
        if control.is_some() && element.children.is_empty() && has_spread_attribute(opening) {
            self.matches.push((
                opening.span.start,
                Kind::Unanalyzable(UnanalyzableReason::ChildSpread),
            ));
        }
        let through = match tag {
            Some("a" | "button" | "div" | "em" | "span" | "strong") => next_ancestor(control, ancestor),
            _ => None,
        };
        self.visit_jsx_opening_element(opening);
        self.children(&element.children, through);
        self.ancestor = ancestor;
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        let ancestor = self.ancestor;
        self.children(&fragment.children, ancestor);
        self.ancestor = ancestor;
    }
}
