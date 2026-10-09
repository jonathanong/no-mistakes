use super::{expressions::expression, Expr, QueryAnnotationFileFacts, Step};
use oxc_ast::ast::{
    BindingPattern, Declaration, ImportDeclarationSpecifier, Program, Statement,
    VariableDeclarationKind,
};

pub(super) fn collect(
    program: &Program<'_>,
    source: &str,
    options: &super::super::EmbeddedSqlOptions,
) -> QueryAnnotationFileFacts {
    let mut facts = QueryAnnotationFileFacts::default();
    facts.trusted_tags.insert("sql".into());
    for statement in &program.body {
        if let Statement::ImportDeclaration(import) = statement {
            for specifier in import.specifiers.iter().flatten() {
                let local = match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(value) => {
                        value.local.name.as_str()
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(value) => value.local.name.as_str(),
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(value) => {
                        value.local.name.as_str()
                    }
                };
                facts.trusted_tags.remove(local);
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(value)
                        if import.source.value == "sql-template-strings" =>
                    {
                        facts.trusted_tags.insert(value.local.name.to_string());
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(value)
                        if super::super::embedded::matches_trusted_sql_import(
                            import.source.value.as_str(),
                            value.imported.name().as_str(),
                            &options.trusted_sql_tags,
                        ) =>
                    {
                        facts.trusted_tags.insert(value.local.name.to_string());
                    }
                    _ => {}
                }
            }
        }
    }
    facts.roots = steps(&program.body, source);
    for step in &facts.roots {
        if let Step::Bind(name, value) = step {
            facts.globals.insert(name.clone(), value.clone());
        }
    }
    let reassigned =
        super::super::embedded::walk::resolve::functions::reassigned::ReassignedNames::collect(
            program,
        );
    facts.trusted_tags.retain(|name| !reassigned.contains(name));
    for (name, value) in &mut facts.globals {
        if matches!(value, Expr::Function(_)) && reassigned.contains(name) {
            *value = Expr::Unknown;
        }
    }
    facts
}

pub(super) fn steps(statements: &[Statement<'_>], source: &str) -> Vec<Step> {
    let mut names = super::super::embedded::walk::resolve::vars::hoisted_names(statements)
        .into_keys()
        .collect::<Vec<_>>();
    names.sort();
    let mut steps = names
        .into_iter()
        .map(|name| Step::Bind(name, Expr::Unsupported))
        .collect::<Vec<_>>();
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(value) => bind_function(value, source, &mut steps),
            Statement::VariableDeclaration(value) => bind_variables(value, source, &mut steps),
            Statement::ExportDeclaration(value) => match &value.declaration {
                Declaration::FunctionDeclaration(value) => bind_function(value, source, &mut steps),
                Declaration::VariableDeclaration(value) => {
                    bind_variables(value, source, &mut steps)
                }
                _ => steps.push(Step::Unsupported),
            },
            Statement::ExpressionStatement(value) => {
                let expr = expression(&value.expression, source);
                let mut appended = Vec::new();
                if let Some(name) = append_binding(&expr, &mut appended) {
                    steps.extend(
                        appended
                            .into_iter()
                            .map(|value| Step::Append(name.clone(), value)),
                    );
                    continue;
                }
                if matches!(
                    &value.expression,
                    oxc_ast::ast::Expression::AssignmentExpression(_)
                        | oxc_ast::ast::Expression::UpdateExpression(_)
                ) {
                    steps.push(Step::Unsupported);
                } else {
                    steps.push(Step::Effect(expr));
                }
            }
            Statement::ReturnStatement(value) => steps.push(Step::Return(
                value
                    .argument
                    .as_ref()
                    .map_or(Expr::Unknown, |value| expression(value, source)),
            )),
            Statement::EmptyStatement(_)
            | Statement::ImportDeclaration(_)
            | Statement::ExportNamedDeclaration(_) => {}
            _ => steps.push(Step::Unsupported),
        }
    }
    steps
}

fn append_binding(expr: &Expr, appended: &mut Vec<Expr>) -> Option<String> {
    match expr {
        Expr::Name(name) => Some(name.clone()),
        Expr::Append(base, tail) => {
            let name = append_binding(base, appended)?;
            appended.push((**tail).clone());
            Some(name)
        }
        _ => None,
    }
}

fn bind_function(value: &oxc_ast::ast::Function<'_>, source: &str, steps: &mut Vec<Step>) {
    let id = value
        .id
        .as_ref()
        .expect("successful function declarations have a binding");
    steps.push(Step::Bind(
        id.name.to_string(),
        super::expressions::function_expression(value, source),
    ));
}

fn bind_variables(
    value: &oxc_ast::ast::VariableDeclaration<'_>,
    source: &str,
    steps: &mut Vec<Step>,
) {
    for declaration in &value.declarations {
        let BindingPattern::BindingIdentifier(id) = &declaration.id else {
            steps.push(Step::Unsupported);
            continue;
        };
        let expr = if value.kind == VariableDeclarationKind::Const {
            declaration
                .init
                .as_ref()
                .map_or(Expr::Unknown, |value| expression(value, source))
        } else {
            Expr::Unsupported
        };
        steps.push(Step::Bind(id.name.to_string(), expr));
    }
}
