use super::PostgresParseError;
use sqlparser::ast::{CreateView, Spanned, Statement, Visit, Visitor};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Location, Token, TokenWithSpan, Whitespace};
use std::{collections::BTreeSet, ops::ControlFlow};

/// Located recursive declarations survive compatibility normalization so source
/// projections can validate columns and model the implicit recursive CTE scope.
#[derive(Default)]
pub(crate) struct RecursiveViews(BTreeSet<Location>);

impl RecursiveViews {
    pub(crate) fn contains(&self, view: &CreateView) -> bool {
        self.0.contains(&view.name.span().start)
    }

    pub(crate) fn relocate(&self, from: Location, to: Location, relocated: &mut Self) {
        if self.0.contains(&from) {
            relocated.0.insert(to);
        }
    }

    pub(crate) fn validate(&self, statement: &Statement) -> Result<(), PostgresParseError> {
        if self.0.is_empty() {
            return Ok(());
        }
        struct Validate<'a>(&'a RecursiveViews);
        impl Visitor for Validate<'_> {
            type Break = PostgresParseError;
            fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<Self::Break> {
                if let Statement::CreateView(view) = statement {
                    if self.0.contains(view) && view.columns.is_empty() {
                        return ControlFlow::Break(PostgresParseError {
                            message: "Recursive views require an explicit column-name list".into(),
                        });
                    }
                }
                ControlFlow::Continue(())
            }
        }
        match statement.visit(&mut Validate(self)) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(error) => Err(error),
        }
    }
}

/// sqlparser 0.63 has no recursive-view modifier. Normalize only declaration
/// headers, retaining their name locations and every original source span.
pub(super) fn prepare(tokens: &mut [TokenWithSpan]) -> RecursiveViews {
    let mut result = RecursiveViews::default();
    let mut preceding = [Keyword::NoKeyword; 5];
    let mut previous_index = 0;
    let mut name_next = false;
    for index in 0..tokens.len() {
        let keyword = match &tokens[index].token {
            Token::Whitespace(_) => continue,
            Token::Word(word) if word.quote_style.is_none() => word.keyword,
            _ => Keyword::NoKeyword,
        };
        if name_next {
            result.0.insert(tokens[index].span.start);
            name_next = false;
        }
        if keyword == Keyword::VIEW
            && matches!(
                preceding,
                [_, _, _, Keyword::CREATE, Keyword::RECURSIVE]
                    | [
                        _,
                        Keyword::CREATE,
                        Keyword::OR,
                        Keyword::REPLACE,
                        Keyword::RECURSIVE
                    ]
                    | [
                        _,
                        _,
                        Keyword::CREATE,
                        Keyword::TEMP | Keyword::TEMPORARY,
                        Keyword::RECURSIVE
                    ]
                    | [
                        Keyword::CREATE,
                        Keyword::OR,
                        Keyword::REPLACE,
                        Keyword::TEMP | Keyword::TEMPORARY,
                        Keyword::RECURSIVE
                    ]
            )
        {
            tokens[previous_index].token = Token::Whitespace(Whitespace::Space);
            name_next = true;
        }
        preceding.rotate_left(1);
        preceding[4] = keyword;
        previous_index = index;
    }
    result
}
