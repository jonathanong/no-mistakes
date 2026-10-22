use super::super::CheckFactPlan;
use crate::codebase::rules::nextjs_no_caching::{
    finish_visitor, prepare_visitor, NextjsCachingFinding, NextjsCachingVisitor,
};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::{Collector, TestFacts};
use crate::codebase::storybook::StorybookFileFacts;
use oxc_ast::ast::{
    ArrowFunctionBody, ArrowFunctionExpression, AssignmentExpression, BlockStatement,
    CallExpression, ExportDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, Function,
    FunctionBody, IdentifierReference, ImportDeclaration, ImportExpression, Program, Statement,
    TSImportEqualsDeclaration, VariableDeclaration, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;
use std::path::Path;

pub(super) struct FusedCheckFacts {
    pub dynamic_imports: Option<TestFacts>,
    pub nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub storybook: Option<StorybookFileFacts>,
}

pub(super) fn collect_fused_check_program(
    path: &Path,
    source: &str,
    program: &Program<'_>,
    plan: &CheckFactPlan,
) -> FusedCheckFacts {
    let storybook = plan.storybook
        && crate::codebase::check_facts::family_scope::applies(&plan.storybook_paths, path);
    if !plan.dynamic_imports && !plan.nextjs_caching && !storybook {
        return FusedCheckFacts {
            dynamic_imports: None,
            nextjs_caching: None,
            storybook: None,
        };
    }
    let mut visitor = CheckProgramVisitor {
        dynamic: plan
            .dynamic_imports
            .then(|| Collector::new(source, program)),
        nextjs: plan
            .nextjs_caching
            .then(|| prepare_visitor(path, source, program)),
        storybook_ids: storybook.then(HashSet::new),
    };
    visitor.visit_program(program);
    FusedCheckFacts {
        dynamic_imports: visitor.dynamic.map(Collector::into_facts),
        nextjs_caching: visitor.nextjs.map(finish_visitor),
        storybook: visitor.storybook_ids.map(|ids| {
            crate::codebase::storybook::extract_program_with_references(source, program, &ids)
        }),
    }
}

struct CheckProgramVisitor<'a> {
    dynamic: Option<Collector<'a>>,
    nextjs: Option<NextjsCachingVisitor<'a>>,
    storybook_ids: Option<HashSet<&'a str>>,
}

impl<'a> Visit<'a> for CheckProgramVisitor<'a> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(ids) = &mut self.storybook_ids {
            ids.insert(ident.name.as_str());
        }
        walk::walk_identifier_reference(self, ident);
    }

    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_import(import);
        }
        walk::walk_import_declaration(self, import);
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Some(dynamic) = &mut self.dynamic {
            dynamic.record_import_expression(import);
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let Some(dynamic) = &mut self.dynamic {
            dynamic.record_variable_declarator(declarator);
        }
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        if let Some(dynamic) = &mut self.dynamic {
            dynamic.record_import_equals(declaration);
        }
        walk::walk_ts_import_equals_declaration(self, declaration);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(dynamic) = &mut self.dynamic {
            dynamic.record_call_expression(call);
        }
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_call(call);
            nextjs.check_fetch_call(call);
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_export_specifiers(export);
        }
        walk::walk_export_named_declaration(self, export);
    }

    fn visit_export_declaration(&mut self, export: &ExportDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_export(export);
        }
        walk::walk_export_declaration(self, export);
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_default_export(export);
        }
        walk::walk_export_default_declaration(self, export);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if let Some(nextjs) = &mut self.nextjs {
            let statements = function
                .body
                .as_ref()
                .map(|body| body.statements.as_slice());
            nextjs.open_function_scope(&function.params, statements);
        }
        walk::walk_function(self, function, flags);
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.pop_const_scope();
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.open_function_scope(&arrow.params, arrow_statements(&arrow.body));
        }
        walk::walk_arrow_function_expression(self, arrow);
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.pop_const_scope();
        }
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_function_body_directives(body);
        }
        walk::walk_function_body(self, body);
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.push_const_scope();
            nextjs.precollect_statements(&block.body);
        }
        walk::walk_block_statement(self, block);
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.pop_const_scope();
        }
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.record_const_declaration(declaration);
        }
        walk::walk_variable_declaration(self, declaration);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_assignment(assignment);
        }
        walk::walk_assignment_expression(self, assignment);
    }
}

fn arrow_statements<'a, 'b>(body: &'b ArrowFunctionBody<'a>) -> Option<&'b [Statement<'a>]> {
    match body {
        ArrowFunctionBody::FunctionBody(body) => Some(body.statements.as_slice()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
