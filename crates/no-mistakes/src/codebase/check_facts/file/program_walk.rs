use super::super::CheckFactPlan;
use crate::codebase::rules::nextjs_no_caching::{
    finish_visitor, prepare_visitor, NextjsCachingFinding, NextjsCachingVisitor,
};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::{Collector, TestFacts};
use crate::codebase::storybook::StorybookFileFacts;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, BlockStatement, CallExpression, CatchClause,
    Class, ExportDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, ForInStatement,
    ForOfStatement, ForStatement, Function, FunctionBody, IdentifierReference, ImportDeclaration,
    ImportExpression, Program, StaticBlock, SwitchStatement, TSImportEqualsDeclaration,
    VariableDeclaration, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;
use std::path::Path;

mod scopes;

pub(super) struct FusedCheckFacts {
    pub dynamic_imports: Option<TestFacts>,
    pub nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub nextjs_caching_low: Option<Vec<NextjsCachingFinding>>,
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
    if !plan.dynamic_imports && !plan.nextjs_caching && !plan.nextjs_caching_low && !storybook {
        return FusedCheckFacts {
            dynamic_imports: None,
            nextjs_caching: None,
            nextjs_caching_low: None,
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
        nextjs_low: plan.nextjs_caching_low.then(|| {
            crate::codebase::rules::nextjs_no_caching::low::Collector::new(path, source, program)
        }),
        storybook_ids: storybook.then(HashSet::new),
    };
    visitor.visit_program(program);
    FusedCheckFacts {
        dynamic_imports: visitor.dynamic.map(Collector::into_facts),
        nextjs_caching: visitor.nextjs.map(finish_visitor),
        nextjs_caching_low: visitor.nextjs_low.map(|collector| collector.finish()),
        storybook: visitor.storybook_ids.map(|ids| {
            crate::codebase::storybook::extract_program_with_references(source, program, &ids)
        }),
    }
}

struct CheckProgramVisitor<'a> {
    dynamic: Option<Collector<'a>>,
    nextjs: Option<NextjsCachingVisitor<'a>>,
    nextjs_low: Option<crate::codebase::rules::nextjs_no_caching::low::Collector<'a>>,
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
        if let Some(low) = &mut self.nextjs_low {
            low.import_declaration(import);
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
        if let Some(low) = &mut self.nextjs_low {
            low.call_expression(call);
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_export_specifiers(export);
        }
        if let Some(low) = &mut self.nextjs_low {
            low.export_named_declaration(export);
        }
        walk::walk_export_named_declaration(self, export);
    }

    fn visit_export_declaration(&mut self, export: &ExportDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_export(export);
        }
        if let Some(low) = &mut self.nextjs_low {
            low.export_declaration(export);
        }
        walk::walk_export_declaration(self, export);
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_default_export(export);
        }
        if let Some(low) = &mut self.nextjs_low {
            low.export_default_declaration(export);
        }
        walk::walk_export_default_declaration(self, export);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.walk_function_scope(function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.walk_arrow_scope(arrow);
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        if let Some(low) = &mut self.nextjs_low {
            low.function_body(body);
        }
        self.walk_function_body_scope(body);
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.walk_block_scope(block);
    }

    fn visit_catch_clause(&mut self, catch: &CatchClause<'a>) {
        self.walk_catch_scope(catch);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.walk_class_scope(class);
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.walk_static_scope(block);
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        self.walk_for_scope(statement);
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        self.walk_for_in_scope(statement);
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.walk_for_of_scope(statement);
    }

    fn visit_switch_statement(&mut self, statement: &SwitchStatement<'a>) {
        self.walk_switch_scope(statement);
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
        if let Some(low) = &mut self.nextjs_low {
            low.assignment_expression(assignment);
        }
        walk::walk_assignment_expression(self, assignment);
    }
}

#[cfg(test)]
mod tests;
