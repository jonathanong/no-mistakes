use oxc_ast::ast::{BindingPattern, Expression, Function, Program, VariableDeclaration};
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;

fn within(node_span: Span, component_span: Span) -> bool {
    node_span.start >= component_span.start && node_span.end <= component_span.end
}

struct DynamicNameCollector<'a> {
    spans: &'a [Span],
    inner_dynamic: Vec<HashSet<String>>,
    outer_dynamic: Vec<HashSet<String>>,
    inner_non_dynamic: Vec<HashSet<String>>,
}

impl<'a> Visit<'a> for DynamicNameCollector<'a> {
    fn visit_variable_declaration(&mut self, v: &VariableDeclaration<'a>) {
        for i in 0..self.spans.len() {
            collect_from_var_decl(v, within(v.span, self.spans[i]), i, self);
        }
        walk::walk_variable_declaration(self, v);
    }

    fn visit_binding_pattern(&mut self, it: &BindingPattern<'a>) {
        // Track every BindingIdentifier within the component span as a potential
        // shadow of an outer dynamic name (covers function params, destructuring, etc.).
        if let BindingPattern::BindingIdentifier(id) = it {
            let name = id.name.as_ref();
            for i in 0..self.spans.len() {
                if within(id.span, self.spans[i]) {
                    self.inner_non_dynamic[i].insert(name.to_string());
                }
            }
        }
        walk::walk_binding_pattern(self, it);
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        // A function declaration name (e.g. `function Lazy() {}`) inside the component
        // span shadows any outer `const Lazy = dynamic(...)` binding.
        if let Some(id) = &func.id {
            let name = id.name.as_ref();
            for i in 0..self.spans.len() {
                if within(func.span, self.spans[i]) {
                    self.inner_non_dynamic[i].insert(name.to_string());
                }
            }
        }
        walk::walk_function(self, func, flags);
    }
}

fn collect_from_var_decl(
    v: &VariableDeclaration<'_>,
    in_component: bool,
    index: usize,
    collector: &mut DynamicNameCollector<'_>,
) {
    for decl in &v.declarations {
        let BindingPattern::BindingIdentifier(id) = &decl.id else {
            continue;
        };
        let Some(init) = &decl.init else {
            continue;
        };
        let name = id.name.as_ref().to_string();
        if is_dynamic_or_lazy_call(init) {
            if in_component {
                collector.inner_dynamic[index].insert(name);
            } else {
                collector.outer_dynamic[index].insert(name);
            }
        } else if in_component {
            collector.inner_non_dynamic[index].insert(name);
        }
    }
}

pub(crate) fn is_dynamic_or_lazy_call(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };
    is_dynamic_or_lazy_call_by_callee(&call.callee)
}

pub(crate) fn is_dynamic_or_lazy_call_by_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(id) => matches!(id.name.as_ref(), "dynamic" | "lazy"),
        Expression::StaticMemberExpression(m) if matches!(&m.object, Expression::Identifier(obj) if obj.name == "React") => {
            m.property.name.as_ref() == "lazy"
        }
        _ => false,
    }
}

pub(crate) fn collect_dynamic_names_for_spans(
    program: &Program<'_>,
    spans: &[Span],
) -> Vec<HashSet<String>> {
    let n = spans.len();
    let mut collector = DynamicNameCollector {
        spans,
        inner_dynamic: vec![HashSet::new(); n],
        outer_dynamic: vec![HashSet::new(); n],
        inner_non_dynamic: vec![HashSet::new(); n],
    };
    collector.visit_program(program);
    (0..n)
        .map(|i| effective_dynamic_names(&collector, i))
        .collect()
}

fn effective_dynamic_names(collector: &DynamicNameCollector<'_>, i: usize) -> HashSet<String> {
    // Effective dynamic names = inner_dynamic ∪ (outer_dynamic ∖ inner_non_dynamic).
    let mut names = collector.inner_dynamic[i].clone();
    for name in &collector.outer_dynamic[i] {
        if !collector.inner_non_dynamic[i].contains(name) {
            names.insert(name.clone());
        }
    }
    names
}
