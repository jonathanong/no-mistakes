//! Const string arrays that a CSP policy may join, spread, or interpolate.
//!
//! Named imports are not read here. The defining module records its exports
//! during the same fact pass, and policy loading substitutes them.

mod eval;
mod imports;
mod scan;

use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, Program, Statement, VariableDeclarationKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) use eval::RegistryJoin;

enum Binding {
    Local(Vec<String>),
    Imported {
        module: PathBuf,
        export_name: String,
    },
}

pub(super) struct ConstArrays {
    bindings: BTreeMap<String, Binding>,
    /// Export name → local binding name.
    exports: BTreeMap<String, String>,
}

impl ConstArrays {
    pub(super) fn build(program: &Program<'_>, importer: &Path) -> Self {
        let mut arrays = Self {
            bindings: BTreeMap::new(),
            exports: imports::exported_locals(program),
        };
        imports::collect_imports(program, importer, &mut arrays.bindings);
        collect_consts(program, &mut arrays.bindings);
        scan::remove_unsound(program, &mut arrays.bindings);
        arrays
    }

    pub(super) fn exported_strings(&self) -> BTreeMap<String, Vec<String>> {
        let mut out = BTreeMap::new();
        for (export_name, local) in &self.exports {
            if let Some(Binding::Local(strings)) = self.bindings.get(local) {
                out.insert(export_name.clone(), strings.clone());
            }
        }
        out
    }
}

fn collect_consts(program: &Program<'_>, bindings: &mut BTreeMap<String, Binding>) {
    let mut blocked = BTreeSet::new();
    for (name, init) in const_initializers(program) {
        if blocked.contains(name) || bindings.contains_key(name) {
            blocked.insert(name.to_string());
            bindings.remove(name);
            continue;
        }
        if let Some(strings) = string_array(init) {
            bindings.insert(name.to_string(), Binding::Local(strings));
        }
    }
}

fn const_initializers<'a>(program: &'a Program<'a>) -> Vec<(&'a str, &'a Expression<'a>)> {
    let mut found = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                push_const_inits(declaration, &mut found);
            }
            Statement::ExportDeclaration(export) => {
                let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                    continue;
                };
                push_const_inits(declaration, &mut found);
            }
            _ => {}
        }
    }
    found
}

fn push_const_inits<'a>(
    declaration: &'a oxc_ast::ast::VariableDeclaration<'a>,
    found: &mut Vec<(&'a str, &'a Expression<'a>)>,
) {
    if declaration.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &declaration.declarations {
        let Some(name) = binding_name(&declarator.id) else {
            continue;
        };
        let Some(init) = &declarator.init else {
            continue;
        };
        found.push((name, init));
    }
}

fn string_array(expr: &Expression<'_>) -> Option<Vec<String>> {
    let Expression::ArrayExpression(array) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    let mut items = Vec::with_capacity(array.elements.len());
    for element in &array.elements {
        let expr = element.as_expression()?;
        items.push(super::csp_strings::eval_static_string(expr)?.into_owned());
    }
    Some(items)
}

fn binding_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
