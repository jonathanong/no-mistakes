use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, File, ItemMod, Meta};

pub(super) fn cfg_test_lines(parsed: &File) -> Vec<usize> {
    let mut visitor = InlineTestVisitor::default();
    visitor.visit_file(parsed);
    visitor.lines
}

#[derive(Default)]
struct InlineTestVisitor {
    lines: Vec<usize>,
}

impl<'ast> Visit<'ast> for InlineTestVisitor {
    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        if item.content.is_none() {
            return;
        }
        visit::visit_item_mod(self, item);
    }

    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        if is_cfg_test(attr) {
            self.lines.push(attr.span().start().line);
        }
        visit::visit_attribute(self, attr);
    }
}

fn is_cfg_test(attr: &Attribute) -> bool {
    if !attr.path().is_ident("cfg") {
        return false;
    }
    let Meta::List(list) = &attr.meta else {
        return false;
    };
    list.tokens
        .to_string()
        .split_whitespace()
        .collect::<String>()
        == "test"
}
