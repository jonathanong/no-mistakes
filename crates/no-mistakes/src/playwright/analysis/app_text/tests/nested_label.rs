use super::*;
use oxc_ast_visit::{walk, Visit};
use std::collections::HashMap;

#[test]
fn nested_label_controls_recurse_through_elements_and_fragments() {
    let path = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "app-text-targets.tsx",
    ]);
    let root = path.parent().unwrap();
    let source = std::fs::read_to_string(&path).expect("fixture should read");
    let settings = settings();

    let (element_control, fragment_control) =
        crate::playwright::ast::with_program(&path, &source, |program, source| {
            let app = AppTextVisitor {
                root,
                path: &path,
                source,
                settings: &settings,
                scoped_static_identifier_defaults: &[],
                targets: Vec::new(),
                controls_by_id: HashMap::new(),
                pending_labels: Vec::new(),
                texts_by_id: HashMap::new(),
                hidden_depth: 0,
            };
            let mut visitor = NestedLabelVisitor {
                app: &app,
                element_control: false,
                fragment_control: false,
            };
            visitor.visit_program(program);
            (visitor.element_control, visitor.fragment_control)
        })
        .expect("fixture parses");

    assert!(element_control);
    assert!(fragment_control);
}

struct NestedLabelVisitor<'app, 'source> {
    app: &'app AppTextVisitor<'source>,
    element_control: bool,
    fragment_control: bool,
}

impl<'a> Visit<'a> for NestedLabelVisitor<'_, '_> {
    fn visit_jsx_element(&mut self, element: &oxc_ast::ast::JSXElement<'a>) {
        if jsx_element_name(&element.opening_element.name) == Some("label") {
            if let Some(control) = self.app.nested_label_control(&element.children, false) {
                self.element_control |= control
                    .selector_refs
                    .iter()
                    .any(|selector| selector.value == "wrapped-email-nested-input");
                self.fragment_control |= control
                    .selector_refs
                    .iter()
                    .any(|selector| selector.value == "fragment-email-nested-input");
            }
        }
        walk::walk_jsx_element(self, element);
    }
}
