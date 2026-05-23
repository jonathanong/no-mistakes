use super::patterns::{banned_segment_config, single_binding_name};
use oxc_ast::ast::{
    Declaration, ImportDeclarationSpecifier, Program, Statement, VariableDeclaration,
};
use std::collections::HashMap;

pub(super) struct TopLevelBindings {
    pub(super) next_config: HashMap<String, Vec<(u32, String)>>,
    pub(super) segment_config: HashMap<String, String>,
    pub(super) local_fetch: bool,
}

pub(super) fn top_level_bindings(program: &Program<'_>, segment_config: bool) -> TopLevelBindings {
    let mut bindings = TopLevelBindings {
        next_config: HashMap::new(),
        segment_config: HashMap::new(),
        local_fetch: false,
    };
    for statement in &program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                bindings.local_fetch |= imports_fetch(import.specifiers.as_deref().map(|v| &**v));
            }
            Statement::VariableDeclaration(var) => {
                collect_top_level_var(var, &mut bindings, segment_config);
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(var)) = export.declaration.as_ref() {
                    collect_top_level_var(var, &mut bindings, segment_config);
                }
            }
            _ => {}
        }
    }
    bindings
}

fn imports_fetch(specifiers: Option<&[ImportDeclarationSpecifier<'_>]>) -> bool {
    specifiers.is_some_and(|specifiers| {
        specifiers.iter().any(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                spec.local.name.as_str() == "fetch"
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                spec.local.name.as_str() == "fetch"
            }
            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                spec.local.name.as_str() == "fetch"
            }
        })
    })
}

fn collect_top_level_var(
    var: &VariableDeclaration<'_>,
    bindings: &mut TopLevelBindings,
    segment_config: bool,
) {
    for decl in &var.declarations {
        let Some(name) = single_binding_name(&decl.id) else {
            continue;
        };
        if name == "fetch" {
            bindings.local_fetch = true;
        }
        let Some(init) = decl.init.as_ref() else {
            continue;
        };
        let config_findings = super::config::expression_findings(init);
        if !config_findings.is_empty() {
            bindings.next_config.insert(name.clone(), config_findings);
        }
        if segment_config && banned_segment_config(name.as_str(), init) {
            bindings
                .segment_config
                .insert(name.clone(), segment_config_message(&name));
        }
    }
}

fn segment_config_message(name: &str) -> String {
    format!("Next.js `{name}` cache segment config is disabled; remove static caching")
}
