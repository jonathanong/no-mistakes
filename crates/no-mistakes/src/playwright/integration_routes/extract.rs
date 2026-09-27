use super::{
    literals::{literal, route_literal},
    receivers::Receivers,
    RouteOccurrence,
};
use crate::codebase::check_facts::PlaywrightModuleResolution;
use crate::codebase::dependencies::extract::ImportedBinding;
use crate::config::v2::schema::RouteCoverageSource;
use crate::playwright::{
    ast,
    playwright_tests::{TestOccurrence, TestOccurrenceScope, TestStatus},
};
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, CallExpression, Function, IfStatement, Program,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::path::Path;

include!("extract_collector.rs");

impl Collector<'_, '_> {
    fn imported(&self, local: &str) -> Option<&ImportedBinding> {
        let binding = self
            .imports
            .iter()
            .find(|binding| binding.local == local && !binding.is_type_only)?;
        let resolved = self.resolution.resolved_path(&binding.specifier, self.path);
        self.imports
            .iter()
            .filter(|peer| {
                !peer.is_type_only
                    && peer.imported == binding.imported
                    && (peer.specifier == binding.specifier
                        || resolved.as_ref().is_some_and(|resolved| {
                            self.resolution
                                .resolved_path(&peer.specifier, self.path)
                                .as_ref()
                                == Some(resolved)
                        }))
            })
            .all(|peer| self.receivers.unique(&peer.local))
            .then_some(binding)
    }

    fn helper_argument(&self, callee: &[String]) -> Option<usize> {
        let (local, method) = match callee {
            [function] => (function.as_str(), None),
            [receiver, method] => (
                self.receivers.constructor(receiver, &self.scope)?,
                Some(method.as_str()),
            ),
            _ => return None,
        };
        let binding = self.imported(local)?;
        let arguments = self
            .config
            .helpers
            .iter()
            .filter(|helper| {
                helper.export == binding.imported
                    && helper.method.as_deref() == method
                    && self.resolution.strict_modules_match(
                        self.root,
                        &helper.module,
                        &binding.specifier,
                        self.path,
                    )
            })
            .map(|helper| helper.url_argument)
            .collect::<std::collections::BTreeSet<_>>();
        (arguments.len() == 1).then(|| *arguments.first().expect("one helper argument"))
    }

    fn runner_kind(&self, callee: &[String]) -> Option<&str> {
        let local = callee.first()?;
        if callee
            .iter()
            .skip(1)
            .any(|part| !matches!(part.as_str(), "only" | "concurrent" | "sequential"))
        {
            return None;
        }
        let binding = self.imported(local)?;
        (binding.specifier == "vitest"
            && matches!(binding.imported.as_str(), "test" | "it" | "describe"))
        .then_some(binding.imported.as_str())
    }
}

impl<'a> Visit<'a> for Collector<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let Some(callee) = ast::expression_path(&call.callee) else {
            return walk::walk_call_expression(self, call);
        };
        if let Some(kind) = self.runner_kind(&callee) {
            if self.test.is_some() {
                return;
            }
            let describe = kind == "describe";
            let Some(name) = call.arguments.first().and_then(literal) else {
                return;
            };
            let Some(index) = crate::playwright::playwright_tests::callback_argument_index(call)
            else {
                return;
            };
            if describe {
                self.describes.push(name);
                self.scope.describes.push(call.span.start);
            } else {
                self.test = Some(name);
                self.scope.test = Some(call.span.start);
            }
            match &call.arguments[index] {
                Argument::ArrowFunctionExpression(function) => {
                    walk::walk_arrow_function_expression(self, function)
                }
                Argument::FunctionExpression(function) => {
                    walk::walk_function(self, function, ScopeFlags::Function)
                }
                _ => unreachable!("callback argument is a function"),
            }
            if describe {
                self.describes.pop();
                self.scope.describes.pop();
            } else {
                self.test = None;
                self.scope.test = None;
            }
            return;
        }
        if let Some(name) = &self.test {
            if let Some(url) = self
                .helper_argument(&callee)
                .and_then(|index| call.arguments.get(index))
                .and_then(|argument| route_literal(argument, self.source))
            {
                self.occurrences.push(RouteOccurrence {
                    source: self.config.clone(),
                    occurrence: TestOccurrence {
                        value: url,
                        status: TestStatus::Active,
                        scope: TestOccurrenceScope::Test,
                        test_name: Some(name.clone()),
                        describe_path: self.describes.clone(),
                        line: self.source[..call.span.start as usize]
                            .bytes()
                            .filter(|byte| *byte == b'\n')
                            .count() as u32
                            + 1,
                    },
                });
            }
        }
        walk::walk_call_expression(self, call);
    }

    // Declaring a callback/function does not execute it. Only imported runner
    // registrations above enter a body; conditional registrations fail closed.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
    fn visit_if_statement(&mut self, _: &IfStatement<'a>) {}

    fn visit_for_statement(&mut self, _: &oxc_ast::ast::ForStatement<'a>) {}
    fn visit_for_in_statement(&mut self, _: &oxc_ast::ast::ForInStatement<'a>) {}
    fn visit_for_of_statement(&mut self, _: &oxc_ast::ast::ForOfStatement<'a>) {}
    fn visit_while_statement(&mut self, _: &oxc_ast::ast::WhileStatement<'a>) {}
    fn visit_do_while_statement(&mut self, _: &oxc_ast::ast::DoWhileStatement<'a>) {}
    fn visit_switch_statement(&mut self, _: &oxc_ast::ast::SwitchStatement<'a>) {}

    fn visit_logical_expression(&mut self, expression: &oxc_ast::ast::LogicalExpression<'a>) {
        if self.test.is_some() {
            walk::walk_logical_expression(self, expression);
        }
    }

    fn visit_conditional_expression(
        &mut self,
        expression: &oxc_ast::ast::ConditionalExpression<'a>,
    ) {
        if self.test.is_some() {
            walk::walk_conditional_expression(self, expression);
        }
    }
}
