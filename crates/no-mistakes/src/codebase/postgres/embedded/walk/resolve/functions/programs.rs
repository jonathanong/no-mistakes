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
            .map(|(name, resolvable)| (*name, resolvable.params, resolvable.body)),
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

    /// Literal strings a same-file helper returns for these arguments.
    /// A failed run, or any result that is not a string, abstains.
    pub(crate) fn identifier_clauses(
        &self,
        name: &str,
        args: &[crate::codebase::postgres::embedded::helpers::SiteArg],
    ) -> Option<Vec<String>> {
        use crate::codebase::postgres::embedded::helpers::program::exec::{run, Ret};
        use crate::codebase::postgres::embedded::helpers::sentinel;
        let program = self.programs.get(name).cloned()?;
        let mut callees = SameFile {
            programs: &self.programs,
            depth: 0,
        };
        let outcomes = run(&program, args, None, &mut callees)?;
        let mut texts = Vec::new();
        for outcome in outcomes {
            let Ret::Str(text, _) = outcome.ret else {
                return None;
            };
            if text.contains('\0') || sentinel::contains(&text) {
                return None;
            }
            if !texts.contains(&text) {
                texts.push(text);
            }
        }
        (!texts.is_empty()).then_some(texts)
    }
}

struct SameFile<'a> {
    programs: &'a crate::fx::FxHashMap<
        String,
        std::sync::Arc<crate::codebase::postgres::embedded::helpers::program::Program>,
    >,
    depth: u8,
}

impl crate::codebase::postgres::embedded::helpers::program::exec::Callees for SameFile<'_> {
    fn call(
        &mut self,
        name: &str,
        args: &[crate::codebase::postgres::embedded::helpers::SiteArg],
    ) -> crate::codebase::postgres::embedded::helpers::program::exec::Callee {
        use crate::codebase::postgres::embedded::helpers::program::exec::{
            run, Callee, MAX_CALL_DEPTH,
        };
        if self.depth >= MAX_CALL_DEPTH {
            return Callee::Failed;
        }
        let Some(program) = self.programs.get(name).cloned() else {
            return Callee::Unresolved;
        };
        self.depth += 1;
        let outcomes = run(&program, args, None, self);
        self.depth -= 1;
        outcomes.map_or(Callee::Failed, Callee::Done)
    }
}

impl super::LocalFunctions {
    /// Moves the lowered programs out once the walk is finished.
    pub(crate) fn take_programs(
        &mut self,
    ) -> crate::fx::FxHashMap<
        String,
        std::sync::Arc<crate::codebase::postgres::embedded::helpers::program::Program>,
    > {
        std::mem::take(&mut self.programs)
    }
}
