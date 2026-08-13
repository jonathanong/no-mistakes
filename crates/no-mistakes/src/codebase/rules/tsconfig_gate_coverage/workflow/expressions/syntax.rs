use super::{lexer::Token, StaticExpressionType};

mod result_type;

const MAX_EXPRESSION_NESTING: usize = 256;

pub(super) fn parse(tokens: &[Token]) -> Option<StaticExpressionType> {
    ExpressionSyntax::new(tokens)
        .parse()
        .map(|expression| expression.static_type)
}

pub(super) fn may_produce_mapping(tokens: &[Token]) -> Option<bool> {
    ExpressionSyntax::new(tokens)
        .parse()
        .map(|expression| expression.may_produce_mapping)
}

#[derive(Clone, Copy)]
struct Expression {
    static_type: StaticExpressionType,
    may_produce_mapping: bool,
}

impl Expression {
    const fn scalar(static_type: StaticExpressionType) -> Self {
        Self {
            static_type,
            may_produce_mapping: false,
        }
    }

    const fn dynamic(may_produce_mapping: bool) -> Self {
        Self {
            static_type: StaticExpressionType::Dynamic,
            may_produce_mapping,
        }
    }
}

struct ExpressionSyntax<'a> {
    tokens: &'a [Token],
    index: usize,
    nesting: usize,
}

impl<'a> ExpressionSyntax<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            index: 0,
            nesting: 0,
        }
    }

    fn parse(mut self) -> Option<Expression> {
        let expression = self.parse_or()?;
        (self.index == self.tokens.len()).then_some(expression)
    }

    fn parse_or(&mut self) -> Option<Expression> {
        let mut expression = self.parse_and()?;
        while self.take(Token::Or) {
            let right = self.parse_and()?;
            expression = result_type::logical_result(expression, right);
        }
        Some(expression)
    }

    fn parse_and(&mut self) -> Option<Expression> {
        let mut expression = self.parse_comparison()?;
        while self.take(Token::And) {
            let right = self.parse_comparison()?;
            expression = result_type::logical_result(expression, right);
        }
        Some(expression)
    }

    fn parse_comparison(&mut self) -> Option<Expression> {
        let mut expression = self.parse_unary()?;
        while self.take(Token::Comparison) {
            self.parse_unary()?;
            expression = Expression::scalar(StaticExpressionType::Boolean);
        }
        Some(expression)
    }

    fn parse_unary(&mut self) -> Option<Expression> {
        let mut unary_count = 0;
        while self.take(Token::Bang) {
            unary_count += 1;
            if unary_count > MAX_EXPRESSION_NESTING {
                return None;
            }
        }
        let expression = self.parse_primary()?;
        (unary_count == 0)
            .then_some(expression)
            .or_else(|| Some(Expression::scalar(StaticExpressionType::Boolean)))
    }

    fn parse_primary(&mut self) -> Option<Expression> {
        match self.next()? {
            Token::Boolean => Some(Expression::scalar(StaticExpressionType::Boolean)),
            Token::Number => Some(Expression::scalar(StaticExpressionType::Number)),
            Token::String => Some(Expression::scalar(StaticExpressionType::String)),
            Token::Null => Some(Expression::scalar(StaticExpressionType::Null)),
            Token::Identifier => self.parse_accessors(Expression::dynamic(true)),
            Token::Function(function) => {
                self.take(Token::LeftParen).then_some(())?;
                let arguments = self.parse_arguments()?;
                function
                    .accepts_argument_count(arguments.len())
                    .then_some(())?;
                self.parse_accessors(Expression {
                    static_type: result_type::function_static_type(function, &arguments),
                    may_produce_mapping: function_may_produce_mapping(function, &arguments),
                })
            }
            Token::LeftParen => {
                let expression = self.parse_nested_expression()?;
                self.take(Token::RightParen).then_some(expression)
            }
            _ => None,
        }
    }

    fn parse_accessors(&mut self, mut expression: Expression) -> Option<Expression> {
        loop {
            if self.take(Token::Dot) {
                if !self.take(Token::Identifier) && !self.take(Token::Star) {
                    return None;
                }
                expression = Expression::dynamic(true);
            } else if self.take(Token::LeftBracket) {
                self.parse_nested_expression()?;
                if !self.take(Token::RightBracket) {
                    return None;
                }
                expression = Expression::dynamic(true);
            } else {
                return Some(expression);
            }
        }
    }

    fn parse_arguments(&mut self) -> Option<Vec<Expression>> {
        self.enter_nesting()?;
        let arguments = self.parse_arguments_inner();
        self.nesting -= 1;
        arguments
    }

    fn parse_arguments_inner(&mut self) -> Option<Vec<Expression>> {
        if self.take(Token::RightParen) {
            return Some(Vec::new());
        }
        let mut arguments = Vec::new();
        loop {
            arguments.push(self.parse_or()?);
            if self.take(Token::RightParen) {
                return Some(arguments);
            }
            if !self.take(Token::Comma) {
                return None;
            }
        }
    }

    fn parse_nested_expression(&mut self) -> Option<Expression> {
        self.enter_nesting()?;
        let expression = self.parse_or();
        self.nesting -= 1;
        expression
    }

    fn enter_nesting(&mut self) -> Option<()> {
        (self.nesting < MAX_EXPRESSION_NESTING).then_some(())?;
        self.nesting += 1;
        Some(())
    }

    fn take(&mut self, token: Token) -> bool {
        if self.tokens.get(self.index) == Some(&token) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = *self.tokens.get(self.index)?;
        self.index += 1;
        Some(token)
    }
}

fn function_may_produce_mapping(
    function: super::lexer::Function,
    _arguments: &[Expression],
) -> bool {
    use super::lexer::Function;

    match function {
        Function::FromJson => true,
        Function::Contains
        | Function::StartsWith
        | Function::EndsWith
        | Function::Format
        | Function::Join
        | Function::ToJson
        | Function::HashFiles
        | Function::Success
        | Function::Failure
        | Function::Always
        | Function::Cancelled => false,
    }
}
