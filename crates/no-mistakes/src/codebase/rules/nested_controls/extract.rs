mod props;

use oxc_ast::ast::{
    Expression, JSXAttribute, JSXAttributeValue, JSXChild, JSXElement, JSXExpression, JSXFragment,
    Program,
};
use oxc_ast_visit::Visit;
use props::{Ancestor, ChildrenProp};

pub(crate) use props::{Kind, Location, UnanalyzableReason};

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

impl Collector {
    fn child_expression<'a>(
        &mut self,
        expression: &JSXExpression<'a>,
        span_start: u32,
        ancestor: Option<Ancestor>,
    ) {
        let Some(expr) = expression.as_expression() else {
            return;
        };
        let inner = expr.get_inner_expression();
        match inner {
            Expression::JSXElement(element) => {
                self.ancestor = ancestor;
                self.visit_jsx_element(element);
            }
            Expression::JSXFragment(fragment) => {
                self.ancestor = ancestor;
                self.visit_jsx_fragment(fragment);
            }
            _ if props::is_safe_literal_child(inner) => {
                self.ancestor = None;
                self.visit_expression(expr);
            }
            _ => {
                if ancestor.is_some() {
                    self.matches.push((
                        span_start,
                        Kind::Unanalyzable(UnanalyzableReason::ChildExpression),
                    ));
                }
                self.ancestor = None;
                self.visit_expression(expr);
            }
        }
    }

    fn children_attribute<'a>(&mut self, attr: &JSXAttribute<'a>, ancestor: Option<Ancestor>) {
        match &attr.value {
            None | Some(JSXAttributeValue::StringLiteral(_)) => {}
            Some(JSXAttributeValue::Element(el)) => {
                self.ancestor = ancestor;
                self.visit_jsx_element(el);
            }
            Some(JSXAttributeValue::Fragment(frag)) => {
                self.ancestor = ancestor;
                self.visit_jsx_fragment(frag);
            }
            Some(JSXAttributeValue::ExpressionContainer(c)) => {
                self.child_expression(&c.expression, c.span.start, ancestor);
            }
        }
    }

    fn children<'a>(&mut self, children: &[JSXChild<'a>], ancestor: Option<Ancestor>) {
        for child in children {
            match child {
                JSXChild::Element(el) => {
                    self.ancestor = ancestor;
                    self.visit_jsx_element(el);
                }
                JSXChild::Fragment(frag) => {
                    self.ancestor = ancestor;
                    self.visit_jsx_fragment(frag);
                }
                JSXChild::Text(_) => {}
                JSXChild::ExpressionContainer(c) => {
                    self.child_expression(&c.expression, c.span.start, ancestor);
                }
                JSXChild::Spread(spread) => {
                    if ancestor.is_some() {
                        self.matches.push((
                            spread.span.start,
                            Kind::Unanalyzable(UnanalyzableReason::ChildSpread),
                        ));
                    }
                    self.ancestor = None;
                    self.visit_jsx_spread_child(spread);
                }
            }
        }
    }

    fn check_nested(
        &mut self,
        span_start: u32,
        tag: Option<&str>,
        control: Option<Ancestor>,
        ancestor: Option<Ancestor>,
    ) -> bool {
        let Some(outer) = ancestor else {
            return false;
        };
        match (control, outer) {
            (Some(Ancestor::Definite(inner)), Ancestor::Definite(outer)) => {
                self.matches
                    .push((span_start, Kind::Nested { inner, outer }));
                true
            }
            (Some(_), _) => {
                self.matches.push((
                    span_start,
                    Kind::Unanalyzable(UnanalyzableReason::UncertainHref),
                ));
                true
            }
            (None, _) if !props::is_allowed_wrapper(tag) => {
                let reason = UnanalyzableReason::WrapperTag(tag.map(str::to_owned));
                self.matches.push((span_start, Kind::Unanalyzable(reason)));
                true
            }
            (None, _) => false,
        }
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        let opening = &element.opening_element;
        let ancestor = self.ancestor.take();
        let tag = props::tag_name(opening);
        // SVG and MathML use a different native element namespace.
        if matches!(tag, Some("svg" | "math")) {
            self.ancestor = ancestor;
            return;
        }
        let control = props::classify_control(tag, opening);
        let reported_opening = self.check_nested(opening.span.start, tag, control, ancestor);
        let through = props::is_allowed_wrapper(tag)
            .then(|| props::next_ancestor(control, ancestor))
            .flatten();
        let mut handled_children_attr = None;
        if through.is_some() && !props::has_effective_jsx_children(&element.children) {
            match props::children_prop(opening) {
                ChildrenProp::Absent => {}
                ChildrenProp::Spread if !reported_opening => {
                    self.matches.push((
                        opening.span.start,
                        Kind::Unanalyzable(UnanalyzableReason::ChildSpread),
                    ));
                }
                ChildrenProp::Spread => {}
                ChildrenProp::Attribute(index, attr) => {
                    handled_children_attr = Some(index);
                    self.children_attribute(attr, through);
                }
            }
        }
        for (index, item) in opening.attributes.iter().enumerate() {
            if handled_children_attr != Some(index) {
                self.ancestor = None;
                self.visit_jsx_attribute_item(item);
            }
        }
        self.children(&element.children, through);
        self.ancestor = ancestor;
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        let ancestor = self.ancestor;
        self.children(&fragment.children, ancestor);
        self.ancestor = ancestor;
    }
}
