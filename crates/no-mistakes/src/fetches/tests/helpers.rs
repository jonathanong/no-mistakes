use oxc_ast::ast::Argument;

pub(crate) fn first_call_expression<'a>(
    statement: &'a oxc_ast::ast::Statement<'a>,
) -> &'a oxc_ast::ast::CallExpression<'a> {
    let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) = statement else {
        panic!("expected expression statement");
    };
    let oxc_ast::ast::Expression::CallExpression(call) = &expr_stmt.expression else {
        panic!("expected call expression");
    };
    call
}

pub(crate) fn first_statement_assignment_call_expression<'a>(
    statement: &'a oxc_ast::ast::Statement<'a>,
) -> &'a oxc_ast::ast::CallExpression<'a> {
    let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) = statement else {
        panic!("expected expression statement");
    };
    let oxc_ast::ast::Expression::AssignmentExpression(assignment) = &expr_stmt.expression else {
        panic!("expected assignment expression");
    };
    let oxc_ast::ast::Expression::CallExpression(call) = &assignment.right else {
        panic!("expected cache wrapper call expression");
    };
    call
}

pub(crate) fn object_argument_from_call_expression<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> &'a oxc_ast::ast::ObjectExpression<'a> {
    let Argument::ObjectExpression(obj) = &call.arguments[1] else {
        panic!("expected object argument");
    };
    obj
}

pub(crate) static RUN_ARGS_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) struct RunArgsEnvGuard {
    pub(crate) _guard: Option<std::sync::MutexGuard<'static, ()>>,
    pub(crate) previous: Option<std::ffi::OsString>,
}

impl Drop for RunArgsEnvGuard {
    fn drop(&mut self) {
        const ENV_VAR: &str = "FETCHES_TEST_ARGS";
        match self.previous.clone() {
            Some(previous) => std::env::set_var(ENV_VAR, previous),
            None => std::env::remove_var(ENV_VAR),
        }
    }
}

impl RunArgsEnvGuard {
    pub(crate) fn release(mut self) -> std::sync::MutexGuard<'static, ()> {
        const ENV_VAR: &str = "FETCHES_TEST_ARGS";
        match self.previous.take() {
            Some(previous) => std::env::set_var(ENV_VAR, previous),
            None => std::env::remove_var(ENV_VAR),
        }
        let guard = self._guard.take().unwrap();
        std::mem::forget(self);
        guard
    }
}

pub(crate) fn with_run_args_env(
    next_value: Option<String>,
    existing: Option<String>,
) -> RunArgsEnvGuard {
    let _guard = RUN_ARGS_MUTEX.lock().unwrap_or_else(|err| err.into_inner());
    const ENV_VAR: &str = "FETCHES_TEST_ARGS";
    let previous: Option<std::ffi::OsString> = match existing {
        Some(existing) => {
            std::env::set_var(ENV_VAR, &existing);
            Some(existing.into())
        }
        None => {
            std::env::remove_var(ENV_VAR);
            None
        }
    };
    match next_value {
        Some(next_value) => std::env::set_var(ENV_VAR, &next_value),
        None => std::env::remove_var(ENV_VAR),
    }

    RunArgsEnvGuard {
        _guard: Some(_guard),
        previous,
    }
}
