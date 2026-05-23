use super::patterns::{banned_segment_config, single_binding_name};
use oxc_ast::ast::{Declaration, Program, Statement, VariableDeclaration};
use std::collections::HashMap;

pub(super) struct TopLevelBindings {
    pub(super) next_config: HashMap<String, Vec<(u32, String)>>,
    pub(super) segment_config: HashMap<String, (u32, String)>,
}

pub(super) fn top_level_bindings(program: &Program<'_>) -> TopLevelBindings {
    let mut bindings = TopLevelBindings {
        next_config: HashMap::new(),
        segment_config: HashMap::new(),
    };
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(var) => collect_top_level_var(var, &mut bindings),
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(var)) = export.declaration.as_ref() {
                    collect_top_level_var(var, &mut bindings);
                }
            }
            _ => {}
        }
    }
    bindings
}

fn collect_top_level_var(var: &VariableDeclaration<'_>, bindings: &mut TopLevelBindings) {
    for decl in &var.declarations {
        let Some(name) = single_binding_name(&decl.id) else {
            continue;
        };
        let Some(init) = decl.init.as_ref() else {
            continue;
        };
        let config_findings = super::config::expression_findings(init);
        if !config_findings.is_empty() {
            bindings.next_config.insert(name.clone(), config_findings);
        }
        if banned_segment_config(name.as_str(), init) {
            bindings.segment_config.insert(
                name.clone(),
                (decl.span.start, segment_config_message(&name)),
            );
        }
    }
}

fn segment_config_message(name: &str) -> String {
    format!("Next.js `{name}` cache segment config is disabled; remove static caching")
}
