pub fn walk_program<V: Visitor>(program: &Program, v: &mut V) {
    for stmt in &program.body {
        walk_statement(stmt, v);
    }
}

fn walk_statement(stmt: &Statement, v: &mut dyn Visitor) {
    match stmt {
        Statement::ImportDeclaration(import) => {
            v.visit_import(import);
        }
        Statement::ExpressionStatement(e) => walk_expression(&e.expression, v),
        Statement::ReturnStatement(r) => {
            walk_optional_expression(r.argument.as_ref(), v);
        }
        Statement::ThrowStatement(t) => walk_expression(&t.argument, v),
        Statement::BlockStatement(b) => {
            for s in &b.body {
                walk_statement(s, v);
            }
        }
        Statement::IfStatement(i) => {
            walk_expression(&i.test, v);
            walk_statement(&i.consequent, v);
            walk_optional_statement(i.alternate.as_ref(), v);
        }
        Statement::WhileStatement(w) => {
            walk_expression(&w.test, v);
            walk_statement(&w.body, v);
        }
        Statement::DoWhileStatement(d) => {
            walk_statement(&d.body, v);
            walk_expression(&d.test, v);
        }
        Statement::ForStatement(f) => walk_for_stmt(f, v),
        Statement::ForOfStatement(f) => {
            walk_expression(&f.right, v);
            walk_statement(&f.body, v);
        }
        Statement::ForInStatement(f) => {
            walk_expression(&f.right, v);
            walk_statement(&f.body, v);
        }
        Statement::VariableDeclaration(var_decl) => {
            for d in &var_decl.declarations {
                walk_optional_expression(d.init.as_ref(), v);
            }
        }
        Statement::LabeledStatement(l) => walk_statement(&l.body, v),
        Statement::TryStatement(t) => walk_try_stmt(t, v),
        Statement::SwitchStatement(s) => {
            walk_expression(&s.discriminant, v);
            for case in &s.cases {
                if let Some(test) = &case.test {
                    walk_expression(test, v);
                }
                for s in &case.consequent {
                    walk_statement(s, v);
                }
            }
        }
        Statement::FunctionDeclaration(f) => {
            walk_function_body(f.body.as_deref(), v);
        }
        Statement::ClassDeclaration(class) => walk_class_body(&class.body, v),
        Statement::ExportNamedDeclaration(e) => {
            walk_optional_declaration(e.declaration.as_ref(), v);
        }
        Statement::ExportDefaultDeclaration(e) => match &e.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                walk_function_body(f.body.as_deref(), v);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                walk_class_body(&class.body, v)
            }
            other => {
                walk_optional_expression(other.as_expression(), v);
            }
        },
        _ => {}
    }
}

fn walk_for_stmt(f: &ForStatement, v: &mut dyn Visitor) {
    if let Some(init) = &f.init {
        match init {
            ForStatementInit::VariableDeclaration(var_decl) => {
                for d in &var_decl.declarations {
                    walk_optional_expression(d.init.as_ref(), v);
                }
            }
            other => {
                walk_optional_expression(other.as_expression(), v);
            }
        }
    }
    if let Some(test) = &f.test {
        walk_expression(test, v);
    }
    if let Some(update) = &f.update {
        walk_expression(update, v);
    }
    walk_statement(&f.body, v);
}

fn walk_try_stmt(t: &TryStatement, v: &mut dyn Visitor) {
    for s in &t.block.body {
        walk_statement(s, v);
    }
    if let Some(handler) = &t.handler {
        for s in &handler.body.body {
            walk_statement(s, v);
        }
    }
    if let Some(fin) = &t.finalizer {
        for s in &fin.body {
            walk_statement(s, v);
        }
    }
}

fn walk_declaration(decl: &Declaration, v: &mut dyn Visitor) {
    match decl {
        Declaration::VariableDeclaration(var_decl) => {
            for d in &var_decl.declarations {
                walk_optional_expression(d.init.as_ref(), v);
            }
        }
        Declaration::FunctionDeclaration(f) => {
            walk_function_body(f.body.as_deref(), v);
        }
        Declaration::ClassDeclaration(class) => walk_class_body(&class.body, v),
        _ => {}
    }
}

fn walk_function_body(body: Option<&FunctionBody>, v: &mut dyn Visitor) {
    if let Some(body) = body {
        walk_statements(&body.statements, v);
    }
}

fn walk_class_body(body: &ClassBody, v: &mut dyn Visitor) {
    for item in &body.body {
        if let ClassElement::MethodDefinition(method) = item {
            walk_function_body(method.value.body.as_deref(), v);
        }
    }
}

fn walk_statements(statements: &[Statement], v: &mut dyn Visitor) {
    for statement in statements {
        walk_statement(statement, v);
    }
}
