//! Lowers every top-level helper to a [`Program`] once per file.

use super::reassigned::ReassignedNames;
use super::shadows::TagShadows;
use super::Resolvable;
use crate::codebase::postgres::embedded::helpers::program::lower::throwing_names;
use crate::codebase::postgres::embedded::helpers::program::lower::{lower, Body, Ctx};
use crate::codebase::postgres::embedded::helpers::program::Program as HelperProgram;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, Program, Statement, VariableDeclaration,
    VariableDeclarationKind,
};
use std::collections::HashMap;

pub(super) fn lower_all(
    program: &Program<'_>,
    source: &str,
    raw: &HashMap<&str, Resolvable<'_>>,
    shadows: &TagShadows,
    reassigned: &ReassignedNames<'_>,
    builder_types: &std::collections::HashSet<String>,
) -> FxHashMap<String, std::sync::Arc<HelperProgram>> {
    let rebound = |name: &str| shadows.contains(name);
    let throwing = throwing_names(
        raw.iter()
            .map(|(name, resolvable)| (*name, resolvable.body)),
    );
    let ctx = Ctx {
        source,
        tags: shadows.imported_tags(),
        builder_types,
        rebound: &rebound,
        throws: &throwing,
    };
    let mut programs = FxHashMap::default();
    for (name, resolvable) in raw {
        if let Some(lowered) = lower(resolvable.params, Body::Statements(resolvable.body), &ctx) {
            insert(&mut programs, name, lowered);
        }
    }
    for (name, declarator_init) in expression_arrows(program) {
        let Expression::ArrowFunctionExpression(arrow) = unwrap_ts_wrappers(declarator_init) else {
            continue;
        };
        let Some(body) = arrow.body.as_expression() else {
            continue;
        };
        if reassigned.contains(name) || arrow.r#async {
            continue;
        }
        if let Some(lowered) = lower(&arrow.params, Body::Expression(body), &ctx) {
            insert(&mut programs, name, lowered);
        }
    }
    programs
}

fn insert(
    programs: &mut FxHashMap<String, std::sync::Arc<HelperProgram>>,
    name: &str,
    lowered: HelperProgram,
) {
    if lowered.contributes_sql() {
        programs.insert(name.to_string(), std::sync::Arc::new(lowered));
    }
}

/// `const name = (…) => expression` declarations, exported or not.
fn expression_arrows<'a>(program: &'a Program<'a>) -> Vec<(&'a str, &'a Expression<'a>)> {
    let mut found = Vec::new();
    for statement in &program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => Some(declaration),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(declaration) => Some(declaration),
                _ => None,
            },
            _ => None,
        };
        if let Some(declaration) = declaration {
            collect_arrows(declaration, &mut found);
        }
    }
    found
}

fn collect_arrows<'a>(
    declaration: &'a VariableDeclaration<'a>,
    found: &mut Vec<(&'a str, &'a Expression<'a>)>,
) {
    if declaration.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &declaration.declarations {
        if let (BindingPattern::BindingIdentifier(ident), Some(init)) =
            (&declarator.id, &declarator.init)
        {
            found.push((ident.name.as_str(), init));
        }
    }
}

impl super::LocalFunctions {
    /// Whether parameter `index` of same-file helper `name` is typed as a
    /// `SQLStatement`, the only parameter that may receive a caller's builder.
    pub(crate) fn program_takes_builder(&self, name: &str, index: usize) -> bool {
        self.programs
            .get(name)
            .and_then(|program| program.params.get(index))
            .is_some_and(|param| param.builder)
    }

    pub(crate) fn has_program(&self, name: &str) -> bool {
        self.programs.contains_key(name)
    }

    /// Moves the lowered programs out once the walk is finished.
    pub(crate) fn take_programs(&mut self) -> FxHashMap<String, std::sync::Arc<HelperProgram>> {
        std::mem::take(&mut self.programs)
    }
}
