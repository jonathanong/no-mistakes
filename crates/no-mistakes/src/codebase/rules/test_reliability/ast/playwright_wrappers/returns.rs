use oxc_ast::ast::{Expression, Statement};

pub(super) fn collect_returns<'ast>(
    stmts: &'ast [Statement<'ast>],
    out: &mut Vec<&'ast Expression<'ast>>,
) -> bool {
    for stmt in stmts {
        match stmt {
            Statement::ReturnStatement(ret) => {
                let Some(arg) = ret.argument.as_ref() else {
                    return false;
                };
                out.push(arg);
            }
            Statement::BlockStatement(block) if !collect_returns(&block.body, out) => return false,
            Statement::IfStatement(if_stmt)
                if !collect_returns(std::slice::from_ref(&if_stmt.consequent), out)
                    || if_stmt
                        .alternate
                        .as_ref()
                        .is_some_and(|alt| !collect_returns(std::slice::from_ref(alt), out)) =>
            {
                return false;
            }
            Statement::SwitchStatement(_)
            | Statement::TryStatement(_)
            | Statement::ForStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::WhileStatement(_)
            | Statement::DoWhileStatement(_)
            | Statement::LabeledStatement(_) => return false,
            _ => {}
        }
    }
    true
}
