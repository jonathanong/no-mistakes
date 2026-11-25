use oxc_ast::ast::{
    JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement, JSXElementName,
    JSXExpression, JSXFragment, JSXOpeningElement, Program,
};
use oxc_ast_visit::Visit;

pub(crate) struct Location {
    pub(super) line: usize,
    pub(super) inner: &'static str,
    pub(super) outer: &'static str,
}

#[derive(Default)]
struct Collector {
    ancestor: Option<&'static str>,
    matches: Vec<(u32, &'static str, &'static str)>,
}

pub(super) fn collect(source: &str, program: &Program<'_>) -> Vec<Location> {
    let mut collector = Collector::default();
    collector.visit_program(program);
    if collector.matches.is_empty() {
        return Vec::new();
    }
    let newlines = memchr::memchr_iter(b'\n', source.as_bytes()).collect::<Vec<_>>();
    collector.matches.into_iter().map(|(offset, inner, outer)| Location {
        line: newlines.partition_point(|newline| *newline < offset as usize) + 1,
        inner,
        outer,
    }).collect()
}

fn tag_name<'a>(opening: &'a JSXOpeningElement<'_>) -> Option<&'a str> {
    match &opening.name {
        JSXElementName::Identifier(name) => Some(name.name.as_str()),
        _ => None,
    }
}

fn has_literal_href(opening: &JSXOpeningElement<'_>) -> bool {
    for item in opening.attributes.iter().rev() {
        match item {
            JSXAttributeItem::SpreadAttribute(_) => return false,
            JSXAttributeItem::Attribute(attribute)
                if matches!(&attribute.name, JSXAttributeName::Identifier(name) if name.name == "href") =>
            {
                return match &attribute.value {
                    Some(JSXAttributeValue::StringLiteral(_)) => true,
                    Some(JSXAttributeValue::ExpressionContainer(value)) => {
                        matches!(value.expression, JSXExpression::StringLiteral(_))
                    }
                    _ => false,
                };
            }
            _ => {}
        }
    }
    false
}

impl Collector {
    fn children<'a>(&mut self, children: &[JSXChild<'a>], ancestor: Option<&'static str>) {
        for child in children {
            self.ancestor = match child {
                JSXChild::Element(_) | JSXChild::Fragment(_) => ancestor,
                _ => None,
            };
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
            Some("button") => Some("button"),
            Some("a") if has_literal_href(opening) => Some("a"),
            _ => None,
        };
        if let (Some(inner), Some(outer)) = (control, ancestor) {
            self.matches.push((opening.span.start, inner, outer));
        }
        let through = match tag {
            Some("a" | "button" | "div" | "em" | "span" | "strong") => control.or(ancestor),
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
