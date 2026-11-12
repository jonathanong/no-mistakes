use super::*;

impl<'a> Projector<'a, '_> {
    pub(super) fn owned_helper_origin(
        &mut self,
        call: &CallExpression<'a>,
        depth: usize,
    ) -> Option<PathOrigin> {
        let (kind, symbol) = self.getter_kind(call)?;
        let body = match kind {
            AstKind::Function(function)
                if !function.generator && function.params.rest.is_none() =>
            {
                function.body.as_deref()?
            }
            AstKind::VariableDeclarator(declaration) if self.immutable_binding(symbol) => {
                match declaration.init.as_ref()?.get_inner_expression() {
                    Expression::ArrowFunctionExpression(arrow) if arrow.params.rest.is_none() => {
                        arrow.body.as_function_body()?
                    }
                    Expression::FunctionExpression(function)
                        if !function.generator && function.params.rest.is_none() =>
                    {
                        function.body.as_deref()?
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };
        let mut origins = Vec::new();
        (!self.owned_statements(&body.statements, depth + 1, &mut origins)?)
            .then(|| PathOrigin::alternatives(origins))
    }

    fn owned_statements(
        &mut self,
        statements: &[Statement<'a>],
        depth: usize,
        origins: &mut Vec<PathOrigin>,
    ) -> Option<bool> {
        let mut falls_through = true;
        for statement in statements {
            falls_through &= self.owned_statement(statement, depth + 1, origins)?;
        }
        Some(falls_through)
    }

    fn owned_statement(
        &mut self,
        statement: &Statement<'a>,
        depth: usize,
        origins: &mut Vec<PathOrigin>,
    ) -> Option<bool> {
        if depth >= 16 || !self.take() {
            return None;
        }
        match statement {
            Statement::ReturnStatement(returned) => {
                let origin = self.origin(returned.argument.as_ref()?, depth + 1);
                if !(origin.anchor == PathAnchor::Temp
                    && origin.root_tails.is_empty()
                    && !origin.dynamic
                    && !origin.segments.iter().any(|part| part == ".."))
                {
                    return None;
                }
                origins.push(origin);
                Some(false)
            }
            Statement::BlockStatement(block) => {
                self.owned_statements(&block.body, depth + 1, origins)
            }
            Statement::IfStatement(branch) => {
                let consequent = self.owned_statement(&branch.consequent, depth + 1, origins)?;
                let alternate = match &branch.alternate {
                    Some(alternate) => self.owned_statement(alternate, depth + 1, origins)?,
                    None => true,
                };
                Some(consequent || alternate)
            }
            Statement::ForOfStatement(looped) => {
                self.owned_statement(&looped.body, depth + 1, origins)?;
                Some(true)
            }
            Statement::VariableDeclaration(_) | Statement::ExpressionStatement(_) => Some(true),
            _ => None,
        }
    }
}
