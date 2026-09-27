impl Receivers {
    fn with_initialization_context(
        &mut self,
        initialization: bool,
        registration: bool,
        action: impl FnOnce(&mut Self),
    ) {
        let previous = (self.initialization_context, self.registration_context);
        self.initialization_context = initialization;
        self.registration_context = registration;
        action(self);
        (self.initialization_context, self.registration_context) = previous;
    }

    fn initializing_callback(
        &self,
        call: &oxc_ast::ast::CallExpression<'_>,
    ) -> Option<(String, String, usize)> {
        let callee = ast::expression_path(&call.callee)?;
        let local = callee.first()?;
        let kind = self.runner_imports.get(local)?;
        let hook = matches!(kind.as_str(), "beforeEach" | "beforeAll");
        if !self.registration_context
            || (!hook && !matches!(kind.as_str(), "describe" | "it" | "test"))
        {
            return None;
        }
        if (hook && callee.len() != 1)
            || callee
                .iter()
                .skip(1)
                .any(|part| !matches!(part.as_str(), "only" | "concurrent" | "sequential"))
        {
            return None;
        }
        if !hook
            && call
                .arguments
                .first()
                .and_then(super::literals::literal)
                .is_none()
        {
            return None;
        }
        Some((
            local.clone(),
            kind.clone(),
            crate::playwright::playwright_tests::callback_argument_index(call)?,
        ))
    }

    fn visit_initializing_call<'a>(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        let Some((origin, kind, index)) = self.initializing_callback(call) else {
            return walk::walk_call_expression(self, call);
        };
        self.origins.push(origin);
        let previous_scope = self.scope.clone();
        match kind.as_str() {
            "describe" => self.scope.describes.push(call.span.start),
            "it" | "test" => self.scope.test = Some(call.span.start),
            _ => self.scope.hook = Some(call.span.start),
        }
        self.with_initialization_context(true, kind == "describe", |this| {
            match &call.arguments[index] {
                oxc_ast::ast::Argument::ArrowFunctionExpression(function) => {
                    walk::walk_arrow_function_expression(this, function)
                }
                oxc_ast::ast::Argument::FunctionExpression(function) => {
                    walk::walk_function(this, function, oxc_syntax::scope::ScopeFlags::Function)
                }
                _ => unreachable!("callback argument is a function"),
            }
        });
        self.scope = previous_scope;
        self.origins.pop();
    }
}
