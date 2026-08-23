use crate::react_traits::analyze::components::ComponentDef;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression, Program, Statement,
};

pub(crate) fn call_is_use_memo(expr: &oxc_ast::ast::CallExpression<'_>) -> bool {
    memo_callee_name(&expr.callee) == "useMemo"
}

fn memo_callee_name<'a>(callee: &'a Expression<'_>) -> &'a str {
    match callee {
        Expression::Identifier(id) => id.name.as_ref(),
        Expression::StaticMemberExpression(m) if matches!(&m.object, Expression::Identifier(obj) if obj.name == "React") => {
            m.property.name.as_ref()
        }
        _ => "",
    }
}

pub(crate) fn is_wrapped_in_memo(program: &Program<'_>, def: &ComponentDef) -> bool {
    for stmt in &program.body {
        match stmt {
            Statement::ExportDefaultDeclaration(e) if def.name == "default" => {
                if let ExportDefaultDeclarationKind::CallExpression(call) = &e.declaration {
                    if memo_callee_name(&call.callee) == "memo" {
                        return true;
                    }
                }
            }
            Statement::ExportDeclaration(e) if def.name != "default" => {
                if let Declaration::VariableDeclaration(v) = &e.declaration {
                    for d in &v.declarations {
                        if let BindingPattern::BindingIdentifier(id) = &d.id {
                            if id.name.as_ref() == def.name {
                                if let Some(Expression::CallExpression(call)) = &d.init {
                                    if memo_callee_name(&call.callee) == "memo" {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Handles `const Page = memo(...); export default Page;` (def.name == "default",
            // def.span covers the local declarator) and re-export alias cases like
            // `const Foo = memo(...); export { Foo as Bar };` (def.name == "Bar",
            // def.span covers Foo's declarator).
            Statement::VariableDeclaration(v) => {
                for d in &v.declarations {
                    if d.span == def.span {
                        if let Some(Expression::CallExpression(call)) = &d.init {
                            if memo_callee_name(&call.callee) == "memo" {
                                return true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests;
