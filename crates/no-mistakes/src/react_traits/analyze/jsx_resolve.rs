//! Shared JSX component-name resolution used by both the forward children
//! collector (`jsx_children`) and the reverse usages collector (`jsx_callsites`).
//!
//! Resolution maps a rendered JSX name (`<Button />`, `<Ns.Button />`) to the
//! `(file, exported_name)` it refers to, using the file's import table and the
//! set of locally declared/exported components.

use crate::react_traits::analyze::components::{is_class_component, is_component_name};
use crate::react_traits::analyze::import_table::ImportTable;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression, JSXAttributeName,
    JSXElementName, JSXMemberExpression, JSXMemberExpressionObject, Program, Statement,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Extract `(root_identifier, member_suffix)` from a JSX element name.
///
/// Returns `Some(root)` only when the name could denote a component: an
/// uppercase identifier (`<Button />`) or a member expression (`<Ns.Button />`).
/// Host elements (`<div />`) and fragments yield `(None, None)`.
pub(crate) fn element_root_and_suffix(
    name: &JSXElementName<'_>,
) -> (Option<String>, Option<String>) {
    match name {
        JSXElementName::IdentifierReference(id) => {
            let n = id.name.as_ref();
            let root = n
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
                .then(|| n.to_string());
            (root, None)
        }
        JSXElementName::MemberExpression(m) => (Some(member_root(m)), Some(member_suffix(m))),
        _ => (None, None),
    }
}

fn member_root(m: &JSXMemberExpression<'_>) -> String {
    match &m.object {
        JSXMemberExpressionObject::IdentifierReference(id) => id.name.to_string(),
        JSXMemberExpressionObject::MemberExpression(m2) => member_root(m2),
        JSXMemberExpressionObject::ThisExpression(_) => String::new(),
    }
}

fn member_suffix(m: &JSXMemberExpression<'_>) -> String {
    m.property.name.to_string()
}

/// The printable name of a JSX attribute (`variant`, or `ns:attr` for a
/// namespaced attribute name).
pub(crate) fn attribute_name(name: &JSXAttributeName<'_>) -> String {
    match name {
        JSXAttributeName::Identifier(id) => id.name.to_string(),
        JSXAttributeName::NamespacedName(n) => {
            format!("{}:{}", n.namespace.name, n.name.name)
        }
    }
}

/// Resolve a JSX root/suffix to the `(file, exported_name)` it renders.
///
/// Imported names resolve through `import_table`; a namespace import combined
/// with a member suffix (`import * as Ns` + `<Ns.Button />`) resolves to the
/// suffix as the exported name. Otherwise the root may be a locally declared
/// component. Returns `None` when the name does not resolve to a known module
/// or local component.
pub(crate) fn resolve_target(
    root: &str,
    member_suffix: Option<&str>,
    import_table: &ImportTable,
    local_components: &HashMap<String, String>,
    file_path: &Path,
) -> Option<(PathBuf, String)> {
    if let Some(entry) = import_table.get(root) {
        let exported = match member_suffix {
            Some(suffix) if entry.exported_name == "*" => suffix.to_string(),
            _ => entry.exported_name.clone(),
        };
        return Some((entry.resolved_path.clone(), exported));
    }
    local_components
        .get(root)
        .map(|exported| (file_path.to_path_buf(), exported.clone()))
}

/// Map every locally declared/exported component symbol to its exported name so
/// `<LocalComponent />` callsites resolve to the current file.
pub(crate) fn collect_local_components(program: &Program<'_>) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for stmt in &program.body {
        match stmt {
            Statement::ExportDefaultDeclaration(e) => match &e.declaration {
                ExportDefaultDeclarationKind::Identifier(id) => {
                    // `export default Page` — map local symbol "Page" -> "default"
                    map.insert(id.name.as_ref().to_string(), "default".to_string());
                }
                ExportDefaultDeclarationKind::CallExpression(call) => {
                    // `export default memo(Page)` — map wrapped identifier -> "default"
                    if let Some(first_arg) = call.arguments.first() {
                        if let Some(Expression::Identifier(id)) = first_arg.as_expression() {
                            map.insert(id.name.as_ref().to_string(), "default".to_string());
                        }
                    }
                }
                _ => {}
            },
            Statement::ExportNamedDeclaration(e) if e.source.is_none() => {
                if let Some(decl) = &e.declaration {
                    collect_inline_export(decl, &mut map);
                } else {
                    // `export { Foo, Bar as Baz }` — local symbol -> exported name
                    for spec in &e.specifiers {
                        let local = spec.local.name().as_ref().to_string();
                        let exported = spec.exported.name().as_ref().to_string();
                        if is_component_name(&local) {
                            map.insert(local, exported);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    map
}

fn collect_inline_export(decl: &Declaration<'_>, map: &mut HashMap<String, String>) {
    match decl {
        Declaration::FunctionDeclaration(f) if f.id.is_some() => {
            let id = f.id.as_ref().unwrap();
            if is_component_name(id.name.as_ref()) {
                let n = id.name.as_ref().to_string();
                map.insert(n.clone(), n);
            }
        }
        Declaration::VariableDeclaration(v) => {
            for d in &v.declarations {
                if let BindingPattern::BindingIdentifier(id) = &d.id {
                    if is_component_name(id.name.as_ref()) {
                        let n = id.name.as_ref().to_string();
                        map.insert(n.clone(), n);
                    }
                }
            }
        }
        Declaration::ClassDeclaration(c) if c.id.is_some() && is_class_component(c) => {
            let id = c.id.as_ref().unwrap();
            if is_component_name(id.name.as_ref()) {
                let n = id.name.as_ref().to_string();
                map.insert(n.clone(), n);
            }
        }
        _ => {}
    }
}
