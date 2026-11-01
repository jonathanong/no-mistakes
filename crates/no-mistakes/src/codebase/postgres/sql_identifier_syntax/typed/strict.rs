use super::*;
use oxc_ast::ast::{TSTypeQueryExprName, TSTypeReference};
use oxc_span::GetSpan;
mod collect;

pub(super) fn annotations(
    program: &Program<'_>,
    types: &FxHashSet<usize>,
    selection: &Selection,
    bindings: &FxHashMap<Span, usize>,
) -> (FxHashSet<Span>, FxHashSet<usize>) {
    let mut facts = collect::Facts::new(bindings);
    facts.visit_program(program);
    let mut alias_counts = FxHashMap::<usize, usize>::default();
    for (id, _) in &facts.aliases {
        *alias_counts.entry(*id).or_default() += 1;
    }
    facts.aliases.retain(|(id, _)| alias_counts[id] == 1);
    let mut proof = Proof {
        types: types.clone(),
        imported: types,
        values: selection.selected.clone(),
        factories: selection
            .factories
            .difference(&facts.writes)
            .copied()
            .collect(),
    };
    proof.values.retain(|id| !facts.writes.contains(id));
    loop {
        let before = (proof.types.len(), proof.values.len());
        for (id, ty) in &facts.aliases {
            if proof.accepts(ty) {
                proof.types.insert(*id);
            }
        }
        for (id, ty) in &facts.parameters {
            if !facts.writes.contains(id) && proof.accepts(ty) {
                proof.values.insert(*id);
            }
        }
        for (id, init) in &facts.constants {
            if !facts.writes.contains(id) && proof.factories.contains(init) {
                proof.values.insert(*id);
            }
        }
        if before == (proof.types.len(), proof.values.len()) {
            break;
        }
    }
    let possible_values = facts
        .initializers
        .iter()
        .filter_map(|(id, factory)| selection.factories.contains(factory).then_some(*id))
        .chain(proof.values.iter().copied())
        .collect();
    let annotations = facts
        .annotations
        .iter()
        .filter_map(|(at, ty)| proof.accepts(ty).then_some(*at))
        .collect();
    (annotations, possible_values)
}

struct Proof<'a> {
    imported: &'a FxHashSet<usize>,
    types: FxHashSet<usize>,
    values: FxHashSet<usize>,
    factories: FxHashSet<usize>,
}
impl Proof<'_> {
    fn accepts(&self, ty: &Shape) -> bool {
        match ty {
            Shape::Type(id) => self.types.contains(id),
            Shape::Generic(id) => self.imported.contains(id),
            Shape::Value(id) => self.values.contains(id),
            Shape::Factory(id) => self.factories.contains(id),
            Shape::Unknown => false,
        }
    }
}
#[derive(Clone, Copy)]
enum Shape {
    Type(usize),
    Generic(usize),
    Value(usize),
    Factory(usize),
    Unknown,
}
struct Lower<'a> {
    bindings: &'a FxHashMap<Span, usize>,
}
impl Lower<'_> {
    fn shape(&self, ty: &TSType<'_>) -> Shape {
        match ty {
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                    return Shape::Unknown;
                };
                if let Some(id) = self.bindings.get(&span(name.span)) {
                    return if reference.type_arguments.is_none() {
                        Shape::Type(*id)
                    } else {
                        Shape::Generic(*id)
                    };
                }
                if self.builtin(reference, "Awaited") {
                    return self
                        .argument(reference)
                        .map_or(Shape::Unknown, |ty| self.return_type(ty));
                }
                self.return_type(ty)
            }
            TSType::TSTypeQuery(query) if query.type_arguments.is_none() => {
                if let TSTypeQueryExprName::IdentifierReference(name) = &query.expr_name {
                    if let Some(id) = self.bindings.get(&span(name.span)) {
                        return Shape::Value(*id);
                    }
                }
                Shape::Unknown
            }
            _ => Shape::Unknown,
        }
    }
    fn builtin(&self, reference: &TSTypeReference<'_>, expected: &str) -> bool {
        matches!(&reference.type_name, TSTypeName::IdentifierReference(name)
            if name.name == expected && !self.bindings.contains_key(&span(name.span)))
    }
    fn argument<'a>(&self, reference: &'a TSTypeReference<'a>) -> Option<&'a TSType<'a>> {
        let arguments = &reference.type_arguments.as_ref()?.params;
        (arguments.len() == 1).then(|| &arguments[0])
    }
    fn return_type(&self, ty: &TSType<'_>) -> Shape {
        let TSType::TSTypeReference(reference) = ty else {
            return Shape::Unknown;
        };
        if !self.builtin(reference, "ReturnType") {
            return Shape::Unknown;
        }
        if let Some(TSType::TSTypeQuery(query)) = self.argument(reference) {
            if query.type_arguments.is_none() {
                if let TSTypeQueryExprName::IdentifierReference(name) = &query.expr_name {
                    if let Some(id) = self.bindings.get(&span(name.span)) {
                        return Shape::Factory(*id);
                    }
                }
            }
        }
        Shape::Unknown
    }
    fn factory_call(&self, expression: &Expression<'_>) -> Option<usize> {
        let expression = unwrap_ts_wrappers(expression);
        let expression = match expression {
            Expression::AwaitExpression(value) => unwrap_ts_wrappers(&value.argument),
            _ => expression,
        };
        if let Expression::CallExpression(call) = expression {
            if let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) {
                return self.bindings.get(&span(name.span)).copied();
            }
        }
        None
    }
}
