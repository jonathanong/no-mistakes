use super::static_values::{
    parameter_environment, program_environment, scope_environment, Environment, Evaluator, Value,
};
use crate::codebase::ts_source::{
    byte_offset_to_line, static_property_key_name, unwrap_ts_wrappers,
};
use oxc_ast::ast::{Expression, MethodDefinition, ObjectProperty, Program, PropertyDefinition};
use oxc_ast_visit::{walk, Visit};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ExtractedDestinations {
    pub(super) body_found: bool,
    pub(super) destinations: Vec<ExtractedDestination>,
    pub(super) saw_destination_property: bool,
    pub(super) incomplete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExtractedDestination {
    pub(super) value: String,
    pub(super) line: usize,
}

pub(super) fn extract_named_destinations(
    path: &Path,
    source: &str,
    name: &str,
) -> ExtractedDestinations {
    crate::ast::with_program(path, source, |program, source| {
        extract_named_destinations_from_program(program, source, name)
    })
    .unwrap_or_default()
}

fn extract_named_destinations_from_program(
    program: &Program<'_>,
    source: &str,
    name: &str,
) -> ExtractedDestinations {
    let mut finder = BodyFinder {
        name,
        source,
        body_found: false,
        destinations: BTreeMap::new(),
        saw_destination_property: false,
        incomplete: false,
        environment: program_environment(program),
    };
    finder.visit_program(program);
    ExtractedDestinations {
        body_found: finder.body_found,
        destinations: finder
            .destinations
            .into_iter()
            .map(|(value, line)| ExtractedDestination { value, line })
            .collect(),
        saw_destination_property: finder.saw_destination_property,
        incomplete: finder.incomplete,
    }
}

struct BodyFinder<'a, 'n> {
    name: &'n str,
    source: &'a str,
    body_found: bool,
    destinations: BTreeMap<String, usize>,
    saw_destination_property: bool,
    incomplete: bool,
    environment: Environment,
}

impl BodyFinder<'_, '_> {
    fn collect_from_expression(&mut self, expression: &Expression<'_>) -> bool {
        let mut evaluator = Evaluator::new();
        let value = match unwrap_ts_wrappers(expression) {
            Expression::FunctionExpression(function) => {
                evaluator.function(function, &self.environment)
            }
            Expression::ArrowFunctionExpression(function) => {
                evaluator.arrow(function, &self.environment, 0)
            }
            _ => return false,
        };
        self.body_found = true;
        self.collect(value);
        true
    }
    fn collect(&mut self, value: Value) {
        match value {
            Value::Array(values) => {
                for value in values.iter().cloned() {
                    self.collect(value);
                }
            }
            Value::Object(properties, complete) => {
                let mut properties = (*properties).clone();
                self.incomplete |= !complete;
                if let Some((destination, offset)) = properties.remove("destination") {
                    self.saw_destination_property = true;
                    if let Value::String(value) = destination {
                        let line = byte_offset_to_line(self.source, offset as usize) as usize;
                        self.destinations.entry(value).or_insert(line);
                    } else {
                        self.incomplete = true;
                    }
                } else if self.name == "rewrites" {
                    for key in ["beforeFiles", "afterFiles", "fallback"] {
                        if let Some((value, _)) = properties.remove(key) {
                            self.collect(value);
                        }
                    }
                    self.incomplete |= !properties.is_empty();
                } else {
                    self.incomplete = true;
                }
            }
            _ => self.incomplete = true,
        }
    }
}

impl<'a> Visit<'a> for BodyFinder<'a, '_> {
    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        let outer = self.environment.clone();
        self.environment = parameter_environment(&function.params, &outer);
        walk::walk_function(self, function, flags);
        self.environment = outer;
    }
    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        let outer = self.environment.clone();
        self.environment = parameter_environment(&function.params, &outer);
        walk::walk_arrow_function_expression(self, function);
        self.environment = outer;
    }
    fn visit_function_body(&mut self, body: &oxc_ast::ast::FunctionBody<'a>) {
        let outer = self.environment.clone();
        self.environment = scope_environment(&body.statements, &outer);
        walk::walk_function_body(self, body);
        self.environment = outer;
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        let outer = self.environment.clone();
        self.environment = scope_environment(&block.body, &outer);
        walk::walk_block_statement(self, block);
        self.environment = outer;
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if self.body_found {
            return;
        }
        if static_property_key_name(&property.key) == Some(self.name)
            && self.collect_from_expression(&property.value)
        {
            return;
        }
        walk::walk_object_property(self, property);
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        if self.body_found {
            return;
        }
        if static_property_key_name(&method.key) == Some(self.name) {
            self.body_found = true;
            let value = Evaluator::new().function(&method.value, &self.environment);
            self.collect(value);
            return;
        }
        walk::walk_method_definition(self, method);
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        if self.body_found {
            return;
        }
        if static_property_key_name(&property.key) == Some(self.name) {
            if let Some(value) = property.value.as_ref() {
                if self.collect_from_expression(value) {
                    return;
                }
            }
        }
        walk::walk_property_definition(self, property);
    }
}
