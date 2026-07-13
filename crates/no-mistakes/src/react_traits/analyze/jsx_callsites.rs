//! Collect every JSX callsite in a file together with the props passed and the
//! `(file, exported_name)` it renders. Used by the reverse `react usages` query
//! to find where a component is rendered. Unlike `jsx_children`, this is not
//! restricted to a single component span — it scans the whole file.

use crate::react_traits::analyze::import_table::ImportTable;
use crate::react_traits::analyze::jsx_resolve::{
    attribute_name, collect_local_components, element_root_and_suffix, resolve_target,
};
use oxc_ast::ast::{JSXAttributeItem, JSXOpeningElement, Program};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashMap;
use std::path::PathBuf;

/// A resolved JSX render site before it is filtered against a target component.
#[derive(Clone)]
pub(crate) struct RawCallsite {
    pub(crate) resolved_path: PathBuf,
    pub(crate) exported_name: String,
    pub(crate) line: usize,
    pub(crate) props: Vec<String>,
    pub(crate) has_spread: bool,
}

struct CallsiteVisitor<'a> {
    import_table: &'a ImportTable,
    local_components: &'a HashMap<String, String>,
    file_path: &'a PathBuf,
    source: &'a str,
    callsites: Vec<RawCallsite>,
}

impl<'a> Visit<'a> for CallsiteVisitor<'a> {
    fn visit_jsx_opening_element(&mut self, elem: &JSXOpeningElement<'a>) {
        let (root_name, member_suffix) = element_root_and_suffix(&elem.name);
        if let Some(root) = root_name {
            if let Some((resolved_path, exported_name)) = resolve_target(
                &root,
                member_suffix.as_deref(),
                self.import_table,
                self.local_components,
                self.file_path,
            ) {
                let line = crate::codebase::ts_source::line_number(self.source, elem.span.start);
                let mut props = Vec::new();
                let mut has_spread = false;
                for attr in &elem.attributes {
                    match attr {
                        JSXAttributeItem::Attribute(a) => props.push(attribute_name(&a.name)),
                        JSXAttributeItem::SpreadAttribute(_) => has_spread = true,
                    }
                }
                self.callsites.push(RawCallsite {
                    resolved_path,
                    exported_name,
                    line,
                    props,
                    has_spread,
                });
            }
        }
        walk::walk_jsx_opening_element(self, elem);
    }
}

pub(crate) fn collect_jsx_callsites(
    program: &Program<'_>,
    import_table: &ImportTable,
    file_path: &PathBuf,
    source: &str,
) -> Vec<RawCallsite> {
    let local_components = collect_local_components(program);
    let mut visitor = CallsiteVisitor {
        import_table,
        local_components: &local_components,
        file_path,
        source,
        callsites: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.callsites
}
