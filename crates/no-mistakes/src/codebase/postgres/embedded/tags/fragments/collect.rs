use super::{yields_fragment, SqlTagNames};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, AssignmentTarget, BindingPattern, Expression,
    FormalParameter, Function, Program, ReturnStatement, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;

// The flag distinguishes a fragment value from a function returning one.
type Key = (bool, String);

/// File-wide conservative fragment ownership, including aliases, parameter
/// defaults and fragment-returning helpers. Scope collisions fail closed.
pub(in crate::codebase::postgres::embedded::tags) fn collect_fragment_bindings(
    program: &Program<'_>,
    tags: &SqlTagNames,
) -> (FxHashSet<String>, FxHashSet<String>) {
    let mut collector = FragmentBindings {
        tags,
        found: FxHashSet::default(),
        dependents: FxHashMap::default(),
        helper_names: crate::fx::FxHashMap::default(),
        return_targets: Vec::new(),
    };
    collector.visit_program(program);
    // Build alias edges once, then propagate to closure. A pass cutoff
    // silently reads longer fragment alias chains as bind values.
    let mut pending: Vec<_> = collector.found.iter().cloned().collect();
    while let Some(key) = pending.pop() {
        for dependent in collector.dependents.get(&key).into_iter().flatten() {
            if collector.found.insert(dependent.clone()) {
                pending.push(dependent.clone());
            }
        }
    }
    let mut values = FxHashSet::default();
    let mut functions = FxHashSet::default();
    for (called, name) in collector.found {
        if called {
            functions.insert(name);
        } else {
            values.insert(name);
        }
    }
    (values, functions)
}

struct FragmentBindings<'t> {
    tags: &'t SqlTagNames,
    found: FxHashSet<Key>,
    dependents: FxHashMap<Key, Vec<Key>>,
    helper_names: crate::fx::FxHashMap<u32, String>,
    return_targets: Vec<Option<Key>>,
}

impl FragmentBindings<'_> {
    fn record(&mut self, key: Key, init: &Expression<'_>) {
        let mut dependencies = Vec::new();
        // Shadowing is intentionally unknown in this file-wide projection.
        let direct = yields_fragment(init, &mut |_| false, self.tags, &mut |name, called| {
            dependencies.push((called, name.to_string()));
            false
        });
        if direct {
            self.found.insert(key);
        } else {
            for dependency in dependencies {
                self.dependents
                    .entry(dependency)
                    .or_default()
                    .push(key.clone());
            }
        }
    }

    fn binding(&mut self, name: &str, init: &Expression<'_>) {
        self.record((false, name.to_string()), init);
        match unwrap_ts_wrappers(init) {
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
                self.helper_names
                    .insert(unwrap_ts_wrappers(init).span().start, name.to_string());
            }
            Expression::Identifier(ident) => {
                self.dependents
                    .entry((true, ident.name.to_string()))
                    .or_default()
                    .push((true, name.to_string()));
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for FragmentBindings<'_> {
    fn visit_formal_parameter(&mut self, parameter: &FormalParameter<'a>) {
        if let (BindingPattern::BindingIdentifier(ident), Some(initializer)) =
            (&parameter.pattern, &parameter.initializer)
        {
            self.binding(ident.name.as_str(), initializer);
        }
        walk::walk_formal_parameter(self, parameter);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(ident), Some(init)) =
            (&declarator.id, &declarator.init)
        {
            self.binding(ident.name.as_str(), init);
        }
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment.left {
            self.binding(ident.name.as_str(), &assignment.right);
        }
        walk::walk_assignment_expression(self, assignment);
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if let BindingPattern::AssignmentPattern(default) = pattern {
            if let BindingPattern::BindingIdentifier(ident) = &default.left {
                self.binding(ident.name.as_str(), &default.right);
            }
        }
        walk::walk_binding_pattern(self, pattern);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let name = self
            .helper_names
            .get(&function.span.start)
            .cloned()
            .or_else(|| function.id.as_ref().map(|id| id.name.to_string()));
        // Async and generator calls return Promise/iterator values, not SQL.
        let target = name.filter(|_| !function.r#async && !function.generator);
        self.return_targets.push(target.map(|name| (true, name)));
        walk::walk_function(self, function, flags);
        self.return_targets.pop();
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let key = self
            .helper_names
            .get(&arrow.span.start)
            .cloned()
            .filter(|_| !arrow.r#async)
            .map(|name| (true, name));
        if let (Some(key), Some(expression)) = (&key, arrow.body.as_expression()) {
            self.record(key.clone(), expression);
        }
        self.return_targets.push(key);
        walk::walk_arrow_function_expression(self, arrow);
        self.return_targets.pop();
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if let (Some(Some(key)), Some(argument)) = (self.return_targets.last(), &statement.argument)
        {
            self.record(key.clone(), argument);
        }
        walk::walk_return_statement(self, statement);
    }
}
