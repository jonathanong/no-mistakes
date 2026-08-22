use super::super::CheckFactPlan;
use crate::codebase::rules::nextjs_no_caching::{
    finish_visitor, prepare_visitor, NextjsCachingFinding, NextjsCachingVisitor,
};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::{Collector, TestFacts};
use crate::codebase::storybook::StorybookFileFacts;
use oxc_ast::ast::{
    AssignmentExpression, CallExpression, ExportDeclaration, ExportDefaultDeclaration,
    ExportNamedDeclaration, FunctionBody, IdentifierReference, ImportDeclaration, ImportExpression,
    Program,
};
use oxc_ast_visit::{walk, Visit};
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
    if !plan.dynamic_imports && !plan.nextjs_caching && !plan.storybook {
        return FusedCheckFacts {
            dynamic_imports: None,
            nextjs_caching: None,
            storybook: None,
        };
    }
    let mut visitor = CheckProgramVisitor {
        dynamic: plan.dynamic_imports.then(|| Collector::new(source)),
        nextjs: plan
            .nextjs_caching
            .then(|| prepare_visitor(path, source, program)),
        storybook_ids: plan.storybook.then(HashSet::new),
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

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_function_body_directives(body);
        }
        walk::walk_function_body(self, body);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.check_assignment(assignment);
        }
        walk::walk_assignment_expression(self, assignment);
    }
}

#[cfg(test)]
mod tests;
