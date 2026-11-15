//! `Object.hasOwn` throw-guard over a module-private frozen literal record.
use super::Resolvable;
use crate::codebase::postgres::embedded::helpers::program::{
    Expr, Mark, Param, Program as HelperProgram, Stmt, SwitchArm,
};
use crate::fx::FxHashMap;
use oxc_ast::ast::{self, BindingPattern, Expression, Statement};
use std::collections::HashMap;
use std::sync::Arc;

mod guard;
mod records;
mod refs;
use guard::{has_own, returned_record, throws};
use records::Values;

pub(super) fn install<'a>(
    module: &ast::Program<'a>,
    source: &str,
    raw: &HashMap<&str, Resolvable<'a>>,
    programs: &mut FxHashMap<String, Arc<HelperProgram>>,
) {
    if refs::shadows_object(module) {
        return;
    }
    let records = module_records(module, source);
    for (name, function) in raw {
        // `Object.hasOwn` lowers to a call, so a program may already exist
        // and still return no identifier text. This exact shape replaces it.
        let Some(lowered) = helper(function.params, function.body, &records) else {
            continue;
        };
        programs.insert((*name).to_string(), Arc::new(lowered));
    }
}

fn helper(
    params: &ast::FormalParameters<'_>,
    body: &ast::FunctionBody<'_>,
    records: &[(String, Values)],
) -> Option<HelperProgram> {
    let [Statement::IfStatement(guard), Statement::ReturnStatement(returned)] =
        body.statements.as_slice()
    else {
        return None;
    };
    if guard.alternate.is_some() || !throws(&guard.consequent) {
        return None;
    }
    let (record, key) = has_own(&guard.test)?;
    let (returned_name, returned_key) = returned_record(returned)?;
    if record != returned_name || key != returned_key {
        return None;
    }
    let values = records.iter().find(|(name, _)| name == record)?.1.clone();
    let lowered = simple_params(params)?;
    let index = lowered.iter().position(|param| param.name == key)?;
    match values {
        Values::Strings(pairs) => {
            let name = key.to_string();
            let arms = pairs
                .into_iter()
                .map(|(case, sql)| {
                    let mark = Mark {
                        line: sql.line,
                        origin: sql.origins.first().copied().unwrap_or(0),
                    };
                    SwitchArm {
                        cases: vec![case],
                        value: Expr::Lit(sql),
                        mark,
                    }
                })
                .collect();
            Some(HelperProgram::new(
                lowered,
                vec![Stmt::Switch(Expr::Ident(name), arms, None)],
            ))
        }
        Values::Objects(entries) => {
            let mut program = HelperProgram::new(lowered, Vec::new());
            program.frozen_key = Some(index);
            program.frozen_entries = entries;
            Some(program)
        }
    }
}

fn simple_params(params: &ast::FormalParameters<'_>) -> Option<Vec<Param>> {
    if params.rest.is_some() {
        return None;
    }
    params
        .items
        .iter()
        .map(|item| {
            if item.initializer.is_some() || !item.decorators.is_empty() {
                return None;
            }
            let BindingPattern::BindingIdentifier(id) = &item.pattern else {
                return None;
            };
            Some(Param {
                name: id.name.to_string(),
                builder: false,
                default: None,
            })
        })
        .collect()
}

fn module_records(module: &ast::Program<'_>, source: &str) -> Vec<(String, Values)> {
    let mut records = Vec::new();
    for statement in &module.body {
        let Some((name, init)) = module_const(statement) else {
            continue;
        };
        if exported(module, name) || !refs::references_allowed(module, name) {
            continue;
        }
        let Some(values) = freeze_values(init, source) else {
            continue;
        };
        records.push((name.to_string(), values));
    }
    records
}

fn module_const<'a>(statement: &'a Statement<'a>) -> Option<(&'a str, &'a Expression<'a>)> {
    let Statement::VariableDeclaration(declaration) = statement else {
        return None;
    };
    if !declaration.kind.is_const() || declaration.declarations.len() != 1 {
        return None;
    }
    let item = &declaration.declarations[0];
    let BindingPattern::BindingIdentifier(id) = &item.id else {
        return None;
    };
    Some((id.name.as_str(), item.init.as_ref()?))
}

fn exported(module: &ast::Program<'_>, name: &str) -> bool {
    module.body.iter().any(|statement| {
        let Statement::ExportNamedDeclaration(export) = statement else {
            return false;
        };
        export
            .specifiers
            .iter()
            .any(|spec| crate::ast::module_export_name(&spec.local) == name)
    })
}

fn freeze_values(init: &Expression<'_>, source: &str) -> Option<Values> {
    records::record_values(records::frozen_object(init)?, source)
}
