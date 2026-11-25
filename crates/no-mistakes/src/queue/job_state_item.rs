use crate::fx::FxHashSet;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, IdentifierReference};
use oxc_ast_visit::{walk, Visit};

#[derive(Clone, Copy)]
pub(super) struct ListingItem<'a> {
    pub names: &'a [&'a str],
    pub start: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ItemUse {
    Absent,
    Uncertain,
    Proven,
}

impl ListingItem<'_> {
    pub(super) fn matches(self, visitor: &QueueVisitor<'_>, id: &IdentifierReference<'_>) -> bool {
        let name = id.name.as_str();
        self.names.contains(&name)
            && visitor
                .object_bindings
                .lookup(name, self.start)
                .zip(visitor.object_bindings.lookup(name, id.span.start))
                .is_some_and(|(item, used)| std::ptr::eq(item, used))
    }

    pub(super) fn enqueue_use(
        self,
        visitor: &QueueVisitor<'_>,
        call: &CallExpression<'_>,
    ) -> ItemUse {
        let mut uses_item = ItemReference {
            visitor,
            item: self,
            found: ItemUse::Absent,
            certain: true,
            in_alias: false,
            aliases: Vec::new(),
            visited: FxHashSet::default(),
        };
        for arg in &call.arguments {
            uses_item.visit_argument(arg);
        }
        uses_item.in_alias = true;
        while uses_item.found != ItemUse::Proven {
            let Some(alias) = uses_item.aliases.pop() else {
                break;
            };
            uses_item.certain = alias.certain;
            uses_item.visit_expression(alias.expr);
            if let Some(pattern) = alias.pattern {
                uses_item.visit_binding_pattern(pattern);
            }
        }
        uses_item.found
    }
}

struct Alias<'a> {
    expr: &'a Expression<'a>,
    pattern: Option<&'a BindingPattern<'a>>,
    certain: bool,
}

struct ItemReference<'c, 'b> {
    visitor: &'b QueueVisitor<'c>,
    item: ListingItem<'b>,
    found: ItemUse,
    certain: bool,
    in_alias: bool,
    aliases: Vec<Alias<'c>>,
    visited: FxHashSet<(u32, bool)>,
}

impl<'a> Visit<'a> for ItemReference<'_, '_> {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        let certain = self.certain;
        if self.in_alias && !structural_alias(expr.get_inner_expression()) {
            self.certain = false;
        }
        walk::walk_expression(self, expr);
        self.certain = certain;
    }

    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        if self.found == ItemUse::Proven {
            return;
        }
        if self.item.matches(self.visitor, id) {
            let usage = if self.certain {
                ItemUse::Proven
            } else {
                ItemUse::Uncertain
            };
            self.found = self.found.max(usage);
            return;
        }
        let Some(binding) = self
            .visitor
            .object_bindings
            .lookup(id.name.as_str(), id.span.start)
            .filter(|binding| binding.scope_start >= self.item.start)
        else {
            return;
        };
        let alias = if let Some((pattern, expr)) = binding.destructuring {
            Alias {
                expr,
                pattern: Some(pattern),
                certain: false,
            }
        } else {
            let Some(expr) = binding.init else {
                return;
            };
            Alias {
                expr,
                pattern: None,
                certain: self.certain,
            }
        };
        if self.visited.insert((binding.resolve_at, alias.certain)) {
            self.aliases.push(alias);
        }
    }
}

fn structural_alias(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::Identifier(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
    )
}
