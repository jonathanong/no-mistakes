use super::names::{Names, ValueKind};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
pub(super) fn literal_length(array: &ArrayExpression<'_>) -> Option<usize> {
    (!array
        .elements
        .iter()
        .any(|element| matches!(element, ArrayExpressionElement::SpreadElement(_))))
    .then_some(array.elements.len())
}
pub(super) fn preserve_empty_array(
    kind: Option<ValueKind>,
    init: Option<&Expression<'_>>,
    declaration_kind: VariableDeclarationKind,
) -> Option<ValueKind> {
    let Some(ValueKind::Array { element, .. }) = kind else {
        return kind;
    };
    let empty = declaration_kind == VariableDeclarationKind::Const
        && matches!(init.map(Expression::without_parentheses), Some(Expression::ArrayExpression(array))
            if array.elements.iter().all(|item| matches!(item, ArrayExpressionElement::Elision(_))));
    let length = (declaration_kind == VariableDeclarationKind::Const)
        .then(|| init.map(Expression::without_parentheses))
        .flatten()
        .and_then(|init| match init {
            Expression::ArrayExpression(array) => literal_length(array),
            _ => None,
        });
    Some(ValueKind::Array {
        element,
        empty,
        length,
    })
}

pub(super) fn invalidate(names: &mut Names, program: &Program<'_>) {
    let candidates = names
        .map
        .iter()
        .filter(|(_, values)| {
            !values.is_empty()
                && values.iter().all(|value| {
                    matches!(
                        value,
                        Some(ValueKind::Array {
                            length: Some(_),
                            ..
                        })
                    )
                })
        })
        .map(|(name, _)| name.clone())
        .collect();
    let mut uses = ArrayUses {
        candidates,
        global_array: names.global("Array"),
        ..ArrayUses::default()
    };
    uses.visit_program(program);
    let mut unstable = uses
        .references
        .into_iter()
        .filter(|(span, _)| !uses.safe.contains(span))
        .map(|(_, name)| name)
        .collect::<FxHashSet<_>>();
    loop {
        let before = unstable.len();
        for (left, right) in &uses.aliases {
            if unstable.contains(left) || unstable.contains(right) {
                unstable.insert(left.clone());
                unstable.insert(right.clone());
            }
        }
        if unstable.len() == before {
            break;
        }
    }
    names.unstable_array_lengths = unstable;
}

#[derive(Default)]
struct ArrayUses {
    candidates: FxHashSet<String>,
    references: FxHashMap<u32, String>,
    safe: FxHashSet<u32>,
    aliases: Vec<(String, String)>,
    global_array: bool,
}

impl ArrayUses {
    fn safe_identifier(&mut self, expression: &Expression<'_>) {
        if let Expression::Identifier(id) = expression.without_parentheses() {
            if self.candidates.contains(id.name.as_str()) {
                self.safe.insert(id.span.start);
            }
        }
    }
}

pub(super) fn source_array<'e>(expression: &'e Expression<'_>) -> Option<&'e str> {
    match expression.without_parentheses() {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::TSAsExpression(cast) => source_array(&cast.expression),
        Expression::TSNonNullExpression(inner) => source_array(&inner.expression),
        Expression::CallExpression(call) => {
            let Expression::StaticMemberExpression(member) = call.callee.without_parentheses()
            else {
                return None;
            };
            if member.property.name.as_str() == "from"
                && matches!(&member.object, Expression::Identifier(id) if id.name == "Array")
            {
                return call
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .and_then(source_array);
            }
            matches!(
                member.property.name.as_str(),
                "map"
                    | "filter"
                    | "flatMap"
                    | "flat"
                    | "slice"
                    | "sort"
                    | "toSorted"
                    | "reverse"
                    | "splice"
            )
            .then(|| source_array(&member.object))
            .flatten()
        }
        _ => None,
    }
}

impl<'a> Visit<'a> for ArrayUses {
    fn visit_variable_declarator(&mut self, node: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(id), Some(init)) = (&node.id, node.init.as_ref())
        {
            if self.candidates.contains(id.name.as_str()) {
                if let Some(source) = source_array(init) {
                    if self.candidates.contains(source) {
                        if matches!(init.without_parentheses(), Expression::Identifier(_)) {
                            self.safe_identifier(init);
                        }
                        self.aliases.push((id.name.to_string(), source.to_owned()));
                    }
                }
            }
        }
        walk::walk_variable_declarator(self, node);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() {
            let method = member.property.name.as_str();
            if matches!(
                method,
                "forEach"
                    | "map"
                    | "filter"
                    | "flatMap"
                    | "reduce"
                    | "reduceRight"
                    | "some"
                    | "every"
                    | "find"
                    | "findIndex"
                    | "findLast"
                    | "findLastIndex"
            ) {
                self.safe_identifier(&member.object);
            }
            if self.global_array
                && method == "from"
                && matches!(&member.object, Expression::Identifier(id) if id.name == "Array")
            {
                if let Some(iterable) = call.arguments.first().and_then(Argument::as_expression) {
                    self.safe_identifier(iterable);
                }
            }
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_identifier_reference(&mut self, node: &IdentifierReference<'a>) {
        if self.candidates.contains(node.name.as_str()) {
            self.references
                .insert(node.span.start, node.name.to_string());
        }
    }
}
