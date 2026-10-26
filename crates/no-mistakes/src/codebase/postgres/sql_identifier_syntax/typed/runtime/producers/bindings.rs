use super::*;
use oxc_ast::ast::VariableDeclarator;
mod finish;

// Negative provenance only: a produced receiver never gains the literal contract.
// Index exact lexical IDs once, including forward declarations; cycles terminate
// when each (binding, kind) is visited once. No helper body is interpreted here.
pub(super) struct Bindings<'s> {
    possible: &'s FxHashSet<Span>,
    ids: &'s FxHashMap<Span, usize>,
    factories: FxHashSet<usize>,
    values: FxHashSet<usize>,
    calls: FxHashMap<usize, Vec<usize>>,
    aliases: FxHashMap<usize, Vec<usize>>,
    invocations: Vec<(usize, Span, Span)>,
    references: Vec<(usize, Span)>,
    methods: super::methods::Methods,
    owners: FxHashMap<Span, usize>,
}
impl<'s> Bindings<'s> {
    pub fn new(possible: &'s FxHashSet<Span>, ids: &'s FxHashMap<Span, usize>) -> Self {
        Self {
            possible,
            ids,
            factories: FxHashSet::default(),
            values: FxHashSet::default(),
            calls: FxHashMap::default(),
            aliases: FxHashMap::default(),
            invocations: Vec::new(),
            references: Vec::new(),
            methods: super::methods::Methods::default(),
            owners: FxHashMap::default(),
        }
    }
    pub fn reference(&mut self, at: Span) {
        if let Some(id) = self.ids.get(&at) {
            self.references.push((*id, at));
        }
    }
    pub fn call(&mut self, call: &CallExpression<'_>) {
        self.methods.call(call, self.ids);
        if let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) {
            let at = span(id.span);
            if let Some(id) = self.ids.get(&at) {
                self.invocations.push((*id, span(call.span), at));
            }
        }
    }
    pub fn function(&mut self, function: &Function<'_>) -> Option<usize> {
        if function.body.is_some() && self.return_type(function.return_type.as_deref()) {
            if let Some(id) = function
                .id
                .as_ref()
                .and_then(|id| self.ids.get(&span(id.span)))
            {
                self.factories.insert(*id);
            }
        }
        function
            .id
            .as_ref()
            .and_then(|id| self.ids.get(&span(id.span)).copied())
            .or_else(|| self.owner(span(function.span)))
    }
    pub fn owner(&self, at: Span) -> Option<usize> {
        self.owners.get(&at).copied()
    }
    pub fn returned(&mut self, owner: usize, expression: &Expression<'_>) {
        self.methods.returned(owner, expression, self.ids);
    }
    fn return_type(&self, annotation: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>) -> bool {
        annotation.is_some_and(|ty| possible::contains(self.possible, &ty.type_annotation))
    }
    pub fn variable(&mut self, variable: &VariableDeclarator<'_>) {
        self.methods.variable(variable, self.ids, self.possible);
        let BindingPattern::BindingIdentifier(binding) = &variable.id else {
            return;
        };
        let Some(destination) = self.ids.get(&span(binding.span)).copied() else {
            return;
        };
        let Some(init) = &variable.init else {
            return;
        };
        match unwrap_ts_wrappers(init) {
            Expression::ArrowFunctionExpression(arrow) => {
                self.owners.insert(span(arrow.span), destination);
            }
            Expression::FunctionExpression(function) => {
                self.owners.insert(span(function.span), destination);
            }
            _ => {}
        }
        match unwrap_ts_wrappers(init) {
            Expression::ArrowFunctionExpression(arrow)
                if self.return_type(arrow.return_type.as_deref()) =>
            {
                self.factories.insert(destination);
            }
            Expression::FunctionExpression(function)
                if self.return_type(function.return_type.as_deref()) =>
            {
                self.factories.insert(destination);
            }
            _ => {}
        }
        // Typed declarations are already classified by Typed, including its
        // configured-factory exception. Only the new assertion source is added here.
        if self.asserted(init) {
            self.values.insert(destination);
        }
        let init = unwrap_ts_wrappers(init);
        let init = if let Expression::AwaitExpression(value) = init {
            unwrap_ts_wrappers(&value.argument)
        } else {
            init
        };
        match init {
            Expression::Identifier(id) => {
                if let Some(source) = self.ids.get(&span(id.span)) {
                    self.aliases.entry(*source).or_default().push(destination);
                }
            }
            Expression::CallExpression(call) => {
                if let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) {
                    if let Some(source) = self.ids.get(&span(id.span)) {
                        self.calls.entry(*source).or_default().push(destination);
                    }
                }
            }
            _ => {}
        }
    }
    pub fn asserted(&self, expression: &Expression<'_>) -> bool {
        match expression {
            Expression::TSAsExpression(value) => {
                possible::contains(self.possible, &value.type_annotation)
                    || self.asserted(&value.expression)
            }
            Expression::TSSatisfiesExpression(value) => {
                possible::contains(self.possible, &value.type_annotation)
                    || self.asserted(&value.expression)
            }
            Expression::TSTypeAssertion(value) => {
                possible::contains(self.possible, &value.type_annotation)
                    || self.asserted(&value.expression)
            }
            Expression::ParenthesizedExpression(value) => self.asserted(&value.expression),
            Expression::AwaitExpression(value) => self.asserted(&value.argument),
            _ => false,
        }
    }
}
