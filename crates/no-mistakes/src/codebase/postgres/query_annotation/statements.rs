mod unsupported;
use super::{expressions::expression, Expr, QueryAnnotationFileFacts, Step};
use oxc_ast::ast::{BindingPattern, Declaration, Program, Statement};

pub(super) fn collect(
    program: &Program<'_>,
    source: &str,
    options: &super::super::EmbeddedSqlOptions,
) -> QueryAnnotationFileFacts {
    let mut facts = QueryAnnotationFileFacts {
        trusted_tags: super::trust::collect(program, options),
        legacy_tag_spans: super::trust::legacy_local_tags(program),
        roots: steps(&program.body, source),
        mapped_arguments: super::mapped_arguments::collect(program),
        ..QueryAnnotationFileFacts::default()
    };
    facts.unmodeled_calls = super::coverage::collect(program, source, &facts.roots);
    for step in &facts.roots {
        if let Step::Bind(name, value) | Step::Hoisted(name, value) = step {
            facts.globals.insert(name.clone(), value.clone());
        }
    }
    let reassigned = super::trust::reassigned(program);
    facts.raw_tag_reassigned = reassigned.contains("String");
    facts.trusted_tags.retain(|name| !reassigned.contains(name));
    for (name, value) in &mut facts.globals {
        if reassigned.contains(name) {
            *value = Expr::Unknown;
        }
    }
    facts
}

pub(super) fn steps(statements: &[Statement<'_>], source: &str) -> Vec<Step> {
    let mut declared: Vec<_> = super::trust::declared_names(statements)
        .into_iter()
        .collect();
    declared.sort();
    let mut names = super::super::embedded::walk::resolve::vars::hoisted_names(statements)
        .into_keys()
        .collect::<Vec<_>>();
    names.sort();
    let mut steps = names.into_iter().map(Step::Var).collect::<Vec<_>>();
    steps.push(Step::Reserve(declared));
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(value) => bind_function(value, source, &mut steps),
            Statement::VariableDeclaration(value) => bind_variables(value, source, &mut steps),
            Statement::ExportDeclaration(value) => match &value.declaration {
                Declaration::FunctionDeclaration(value) => bind_function(value, source, &mut steps),
                Declaration::VariableDeclaration(value) => {
                    bind_variables(value, source, &mut steps)
                }
                Declaration::TSTypeAliasDeclaration(_) | Declaration::TSInterfaceDeclaration(_) => {
                }
                _ => steps.push(Step::Unsupported),
            },
            Statement::ExportDefaultDeclaration(value) => {
                steps.extend(super::exports::default_steps(value, source))
            }
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
            | Statement::ExportNamedDeclaration(_)
            | Statement::TSTypeAliasDeclaration(_)
            | Statement::TSInterfaceDeclaration(_) => {}
            _ => {
                steps.push(Step::PotentialCalls(unsupported::calls(statement)));
                steps.push(Step::Unsupported);
            }
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
    steps.push(Step::Hoisted(
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
        // A bare var redeclaration preserves parameters and hoisted functions.
        if value.kind == oxc_ast::ast::VariableDeclarationKind::Var && declaration.init.is_none() {
            continue;
        }
        let BindingPattern::BindingIdentifier(id) = &declaration.id else {
            steps.push(Step::Unsupported);
            continue;
        };
        let init = declaration
            .init
            .as_ref()
            .map_or(Expr::Unknown, |value| expression(value, source));
        // Mutable declaration initializers still execute. Later assignments
        // make the containing helper unsupported instead of hiding these calls.
        steps.push(Step::Bind(id.name.to_string(), init));
    }
}
