use super::ScopeCollector;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{self, Expression, Statement};
use oxc_span::{GetSpan, Span};

// Collected with the existing scoped walk; never rescan an AST at a call site.
// Binding declaration offset, lexical scope, and executing function.
type Binding = (u32, Span, Option<Span>);
// Reference span, executing function, and whether it writes the binding.
type Reference = (Span, Option<Span>, bool);
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Bindings {
    pub(super) enabled: bool,
    declarations: FxHashMap<String, Vec<Binding>>,
    references: FxHashMap<String, Vec<Reference>>,
    assignments: FxHashMap<u32, Span>,
    pub(super) candidates: FxHashSet<u32>,
    pub(super) functions: Vec<Span>,
    pub(super) writing: usize,
}

impl ScopeCollector {
    pub(super) fn declare_delayed_binding(&mut self, id: &ast::BindingIdentifier<'_>) {
        if !self.delayed.enabled {
            return;
        }
        let function = self.delayed.functions.last().copied();
        let scope = *self.scopes.last().expect("program scope");
        let scope = function
            .filter(|owner| scope.start < owner.start || owner.end < scope.end)
            .unwrap_or(scope);
        self.delayed
            .declarations
            .entry(id.name.to_string())
            .or_default()
            .push((id.span.start, scope, function));
    }

    pub(super) fn record_delayed_reference(&mut self, id: &ast::IdentifierReference<'_>) {
        if !self.delayed.enabled {
            return;
        }
        let owner = self.delayed.functions.last().copied();
        let writing = self.delayed.writing > 0;
        self.delayed
            .references
            .entry(id.name.to_string())
            .or_default()
            .push((id.span, owner, writing));
    }

    pub(super) fn record_delayed_statements(&mut self, statements: &[Statement<'_>]) {
        if !self.delayed.enabled {
            return;
        }
        let end = statements
            .last()
            .map_or(0, |statement| statement.span().end);
        for statement in statements {
            let Statement::ExpressionStatement(statement) = statement else {
                continue;
            };
            let Expression::AssignmentExpression(assignment) = &statement.expression else {
                continue;
            };
            let ast::AssignmentTarget::AssignmentTargetIdentifier(id) = &assignment.left else {
                continue;
            };
            if assignment.operator == ast::AssignmentOperator::Assign {
                self.delayed
                    .assignments
                    .insert(id.span.start, Span::new(assignment.span.end, end));
            }
        }
    }

    pub(in super::super) fn finish_delayed_receivers(&mut self) {
        for (name, references) in &self.delayed.references {
            let Some(declarations) = self.delayed.declarations.get(name) else {
                continue;
            };
            for (declaration, _, function) in declarations
                .iter()
                .filter(|(start, _, _)| self.delayed.candidates.contains(start))
            {
                let Some(identity) = self
                    .found
                    .undeclared
                    .get(declaration)
                    .and_then(|names| names.get(name)?.get(""))
                    .filter(|identity| identity.configured_type)
                else {
                    continue;
                };
                // A declaration shadows throughout its scope, including the TDZ.
                let owned = |reference: &&Reference| {
                    declarations
                        .iter()
                        .filter(|(_, scope, _)| {
                            scope.start <= reference.0.start && reference.0.end <= scope.end
                        })
                        .min_by_key(|(_, scope, _)| scope.end - scope.start)
                        .is_some_and(|(start, _, _)| start == declaration)
                };
                let mut writes = references
                    .iter()
                    .filter(owned)
                    .filter(|(_, _, writing)| *writing);
                let Some((write, owner, _)) = writes.next() else {
                    continue;
                };
                if writes.next().is_some() || owner != function || write.start < *declaration {
                    continue;
                }
                let Some(range) = self.delayed.assignments.get(&write.start) else {
                    continue;
                };
                // The unique write ends before this range, so its owned sites are reads.
                for (span, _, _) in references.iter().filter(owned).filter(|(span, owner, _)| {
                    owner == function && range.start <= span.start && span.end <= range.end
                }) {
                    if let Some(owner) = identity.owner {
                        self.spans.push(super::PendingRelativeSpan {
                            owners: vec![owner],
                            name: name.clone(),
                            start: span.start,
                            end: span.end,
                        });
                    } else {
                        self.found.delayed.insert(span.start);
                    }
                }
            }
        }
    }
}
