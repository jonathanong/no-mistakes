use super::*;
use oxc_ast::ast::TSSignature;
use std::collections::VecDeque;

type Key = (usize, Vec<String>);
#[derive(Default)]
pub(super) struct Methods {
    known: FxHashSet<Key>,
    readers: FxHashMap<Key, Vec<usize>>,
    value_readers: FxHashMap<usize, Vec<usize>>,
    value_roots: FxHashMap<usize, Vec<Span>>,
    calls: Vec<(Key, Span, Span)>,
    destinations: Vec<(Key, usize)>,
    references: Vec<(Key, Span)>,
}
impl Methods {
    fn key(expression: &Expression<'_>, ids: &FxHashMap<Span, usize>) -> Option<(Key, Span)> {
        let (root, mut path) = chain(expression)?;
        path.remove(0);
        Some(((*ids.get(&root)?, path), root))
    }
    pub fn variable(
        &mut self,
        variable: &oxc_ast::ast::VariableDeclarator<'_>,
        ids: &FxHashMap<Span, usize>,
        possible: &FxHashSet<Span>,
    ) {
        let BindingPattern::BindingIdentifier(binding) = &variable.id else {
            return;
        };
        let Some(id) = ids.get(&span(binding.span)).copied() else {
            return;
        };
        if let Some(annotation) = &variable.type_annotation {
            if let TSType::TSTypeLiteral(ty) = &annotation.type_annotation {
                for member in &ty.members {
                    if let TSSignature::TSMethodSignature(method) = member {
                        if method
                            .return_type
                            .as_ref()
                            .is_some_and(|ty| possible::contains(possible, &ty.type_annotation))
                        {
                            if let Some(name) = method.key.static_name() {
                                self.known.insert((id, vec![name.to_string()]));
                            }
                        }
                    }
                }
            }
        }
        let Some(init) = variable.init.as_ref().map(unwrap_ts_wrappers) else {
            return;
        };
        if let Expression::CallExpression(call) = init {
            if let Some((key, _)) = Self::key(&call.callee, ids) {
                self.destinations.push((key, id));
            }
        } else if let Some((key, root)) = Self::key(init, ids) {
            self.readers.entry(key.clone()).or_default().push(id);
            self.references.push((key, root));
        }
    }
    pub fn call(&mut self, call: &CallExpression<'_>, ids: &FxHashMap<Span, usize>) {
        if let Some((key, root)) = Self::key(&call.callee, ids) {
            self.calls.push((key, span(call.span), root));
        }
    }
    pub fn returned(
        &mut self,
        owner: usize,
        expression: &Expression<'_>,
        ids: &FxHashMap<Span, usize>,
    ) {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(id) => {
                if let Some(id) = ids.get(&span(id.span)) {
                    self.value_readers.entry(*id).or_default().push(owner);
                    self.value_roots
                        .entry(*id)
                        .or_default()
                        .push(span(expression.get_inner_expression().span()));
                }
            }
            Expression::AwaitExpression(value) => self.returned(owner, &value.argument, ids),
            Expression::CallExpression(call) => {
                if let Some((key, _)) = Self::key(&call.callee, ids) {
                    self.readers.entry(key).or_default().push(owner);
                }
            }
            Expression::LogicalExpression(value) => {
                self.returned(owner, &value.left, ids);
                self.returned(owner, &value.right, ids);
            }
            Expression::ConditionalExpression(value) => {
                self.returned(owner, &value.consequent, ids);
                self.returned(owner, &value.alternate, ids);
            }
            _ => {}
        }
    }
    pub fn returned_roots(&self, id: usize) -> &[Span] {
        self.value_roots.get(&id).map_or(&[], Vec::as_slice)
    }
    pub fn returned_owners(&self, id: usize, factory: bool) -> &[usize] {
        if factory {
            self.readers.get(&(id, Vec::new()))
        } else {
            self.value_readers.get(&id)
        }
        .map_or(&[], Vec::as_slice)
    }
    pub fn finish(
        &self,
        factories: &mut FxHashSet<usize>,
    ) -> (FxHashSet<usize>, FxHashSet<Span>, FxHashSet<Span>) {
        let mut known = self.known.clone();
        known.extend(factories.iter().map(|id| (*id, Vec::new())));
        let mut queue = VecDeque::from_iter(known.iter().cloned());
        while let Some(key) = queue.pop_front() {
            for id in self.readers.get(&key).into_iter().flatten() {
                let next = (*id, Vec::new());
                if known.insert(next.clone()) {
                    queue.push_back(next);
                }
            }
        }
        factories.extend(
            known
                .iter()
                .filter_map(|(id, path)| path.is_empty().then_some(*id)),
        );
        let values = self
            .destinations
            .iter()
            .filter_map(|(key, id)| known.contains(key).then_some(*id))
            .collect();
        let mut uses = FxHashSet::default();
        let mut transports = FxHashSet::default();
        for (key, at, root) in &self.calls {
            if known.contains(key) {
                uses.insert(*at);
                transports.insert(*root);
            }
        }
        for (key, root) in &self.references {
            if known.contains(key) {
                transports.insert(*root);
            }
        }
        (values, uses, transports)
    }
}
