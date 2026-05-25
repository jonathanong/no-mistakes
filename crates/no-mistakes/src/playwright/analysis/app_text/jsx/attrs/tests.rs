use super::*;
use oxc_ast::ast::{JSXElement, JSXOpeningElement};
use oxc_ast_visit::{walk, Visit};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
struct AttrSnapshot {
    label_exists: bool,
    hidden: Option<bool>,
    aria_hidden: Option<bool>,
    size: Option<u32>,
}

#[derive(Default)]
struct AttrVisitor<'a> {
    source: &'a str,
    snapshots: BTreeMap<String, AttrSnapshot>,
}

impl<'a> Visit<'a> for AttrVisitor<'_> {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        if let Some(case) = case_name(&element.opening_element) {
            self.snapshots.insert(
                case,
                AttrSnapshot {
                    label_exists: attr_exists_at_runtime(&element.opening_element, "aria-label"),
                    hidden: bool_attr(&element.opening_element, "hidden"),
                    aria_hidden: aria_bool_attr(&element.opening_element, "aria-hidden"),
                    size: numeric_attr(&element.opening_element, "size", self.source),
                },
            );
        }
        walk::walk_jsx_element(self, element);
    }
}

fn case_name(opening: &JSXOpeningElement<'_>) -> Option<String> {
    find_attr(opening, "data-case").and_then(|attribute| match attribute.value.as_ref()? {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    })
}

fn snapshots() -> BTreeMap<String, AttrSnapshot> {
    let path = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "jsx-attrs-branches.tsx",
    ]);
    let source = std::fs::read_to_string(&path).expect("fixture should read");
    crate::playwright::ast::with_program(Path::new(&path), &source, |program, source| {
        let mut visitor = AttrVisitor {
            source,
            snapshots: BTreeMap::new(),
        };
        visitor.visit_program(program);
        visitor.snapshots
    })
    .expect("fixture parses")
}

#[test]
fn jsx_attr_helpers_cover_static_dynamic_and_ts_wrapped_values() {
    let snapshots = snapshots();
    assert!(snapshots["label-bare"].label_exists);
    assert!(snapshots["label-string"].label_exists);
    assert!(snapshots["label-dynamic"].label_exists);
    assert!(!snapshots["label-null"].label_exists);
    assert!(!snapshots["label-undefined"].label_exists);
    assert!(!snapshots["label-as-null"].label_exists);
    assert!(snapshots["label-non-null"].label_exists);
    assert!(!snapshots["label-satisfies-null"].label_exists);
    assert!(snapshots["label-satisfies"].label_exists);
    assert!(snapshots["label-element"].label_exists);

    assert_eq!(snapshots["hidden-bare"].hidden, Some(true));
    assert_eq!(snapshots["hidden-string"].hidden, Some(true));
    assert_eq!(snapshots["hidden-null"].hidden, Some(false));
    assert_eq!(snapshots["hidden-zero"].hidden, Some(false));
    assert_eq!(snapshots["hidden-string-empty"].hidden, Some(false));
    assert_eq!(snapshots["hidden-template"].hidden, Some(true));
    assert_eq!(snapshots["hidden-undefined"].hidden, Some(false));
    assert_eq!(snapshots["hidden-as"].hidden, Some(true));
    assert_eq!(snapshots["hidden-satisfies"].hidden, Some(false));
    assert_eq!(snapshots["hidden-non-null"].hidden, Some(true));
    assert_eq!(snapshots["hidden-null-wrapped"].hidden, Some(false));
    assert_eq!(snapshots["hidden-zero-wrapped"].hidden, Some(false));
    assert_eq!(snapshots["hidden-string-wrapped"].hidden, Some(true));
    assert_eq!(snapshots["hidden-undefined-wrapped"].hidden, Some(false));
    assert_eq!(snapshots["hidden-non-null-dynamic"].hidden, None);
    assert_eq!(snapshots["hidden-dynamic"].hidden, None);

    assert_eq!(snapshots["aria-string-true"].aria_hidden, Some(true));
    assert_eq!(snapshots["aria-string-false"].aria_hidden, Some(false));
    assert_eq!(snapshots["aria-string-invalid"].aria_hidden, None);
    assert_eq!(snapshots["aria-expr-string-true"].aria_hidden, Some(true));
    assert_eq!(snapshots["aria-expr-string-false"].aria_hidden, Some(false));
    assert_eq!(snapshots["aria-expr-string-invalid"].aria_hidden, None);
    assert_eq!(snapshots["aria-as"].aria_hidden, Some(true));
    assert_eq!(snapshots["aria-satisfies"].aria_hidden, Some(false));
    assert_eq!(snapshots["aria-non-null"].aria_hidden, Some(true));
    assert_eq!(snapshots["aria-non-null-dynamic"].aria_hidden, None);
    assert_eq!(snapshots["aria-element"].aria_hidden, None);

    assert_eq!(snapshots["size-string"].size, Some(3));
    assert_eq!(snapshots["size-number"].size, Some(4));
    assert_eq!(snapshots["size-as"].size, Some(5));
    assert_eq!(snapshots["size-satisfies"].size, Some(6));
    assert_eq!(snapshots["size-non-null"].size, Some(7));
    assert_eq!(snapshots["size-dynamic-wrapped"].size, None);
    assert_eq!(snapshots["size-non-null-dynamic"].size, None);
    assert_eq!(snapshots["size-dynamic"].size, None);
    assert_eq!(snapshots["size-negative"].size, None);
    assert_eq!(snapshots["size-element"].size, None);
}
