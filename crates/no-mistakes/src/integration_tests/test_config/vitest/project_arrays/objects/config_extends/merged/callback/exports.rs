use super::{Expression, Program, Statement};
use oxc_ast::ast::{
    AssignmentTarget, BindingPattern, Declaration, IdentifierReference, SimpleAssignmentTarget,
    VariableDeclarationKind,
};
use oxc_ast_visit::Visit;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn immutable_bindings<'a>(
    program: &'a Program<'a>,
) -> BTreeMap<String, &'a Expression<'a>> {
    let mut writes = Writes::default();
    writes.visit_program(program);
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => declaration,
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(declaration) => declaration,
                _ => continue,
            },
            _ => continue,
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declaration in &declaration.declarations {
            let BindingPattern::BindingIdentifier(identifier) = &declaration.id else {
                continue;
            };
            if !writes.0.contains(identifier.name.as_str()) {
                if let Some(init) = &declaration.init {
                    bindings.insert(identifier.name.to_string(), init);
                }
            }
        }
    }
    bindings
}

pub(super) fn default_expression<'a>(
    program: &'a Program<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> Option<&'a Expression<'a>> {
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                return export.declaration.as_expression()
            }
            Statement::ExportNamedDeclaration(export) if !export.export_kind.is_type() => {
                for specifier in &export.specifiers {
                    if !specifier.export_kind.is_type()
                        && crate::ast::module_export_name(&specifier.exported) == "default"
                    {
                        return bindings
                            .get(crate::ast::module_export_name(&specifier.local).as_ref())
                            .copied();
                    }
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Default)]
struct Writes(BTreeSet<String>);
impl<'a> Visit<'a> for Writes {
    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        TargetNames(&mut self.0).visit_assignment_target(target);
    }
    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        TargetNames(&mut self.0).visit_simple_assignment_target(target);
    }
}

struct TargetNames<'a>(&'a mut BTreeSet<String>);
impl<'a> Visit<'a> for TargetNames<'_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.0.insert(identifier.name.to_string());
    }
}
