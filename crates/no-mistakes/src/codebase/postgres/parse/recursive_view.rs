use super::PostgresParseError;
use sqlparser::ast::{CreateView, Spanned, Statement, Visit, VisitMut, Visitor, VisitorMut};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Location, Token, TokenWithSpan, Whitespace};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

mod query;

/// Located recursive declarations survive compatibility normalization so source
/// projections can validate columns and model the implicit recursive CTE scope.
#[derive(Default)]
pub(crate) struct RecursiveViews {
    names: BTreeSet<Location>,
}

impl RecursiveViews {
    pub(crate) fn contains(&self, view: &CreateView) -> bool {
        self.names.contains(&view.name.span().start)
    }

    pub(crate) fn relocate(&self, from: Location, to: Location, relocated: &mut Self) {
        if self.names.contains(&from) {
            relocated.names.insert(to);
        }
    }

    pub(super) fn restore_statements(
        &self,
        statements: &mut [super::LocatedStatement],
        original_heads: &BTreeMap<Location, String>,
    ) {
        for located in statements {
            if let Some(failure) = &mut located.recovery_failure {
                let location = Location::new(failure.line as u64, failure.column as u64);
                if let Some(head) = original_heads.get(&location) {
                    failure.head.clone_from(head);
                }
            }
            self.restore(&mut located.statement);
        }
    }

    /// Preserve the implicit binding for every downstream AST consumer.
    pub(crate) fn restore(&self, statement: &mut Statement) {
        struct Restore<'a>(&'a RecursiveViews);
        impl VisitorMut for Restore<'_> {
            type Break = ();
            fn post_visit_statement(&mut self, statement: &mut Statement) -> ControlFlow<()> {
                if let Statement::CreateView(view) = statement {
                    if self.0.contains(view) {
                        query::wrap(view);
                    }
                }
                ControlFlow::Continue(())
            }
        }
        if !self.names.is_empty() {
            let _ = statement.visit(&mut Restore(self));
        }
    }

    pub(crate) fn validate(&self, statement: &Statement) -> Result<(), PostgresParseError> {
        if self.names.is_empty() {
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
pub(super) fn prepare(
    tokens: &mut [TokenWithSpan],
    mut original_heads: Option<&mut BTreeMap<Location, String>>,
) -> RecursiveViews {
    let mut result = RecursiveViews::default();
    let mut preceding = [Keyword::NoKeyword; 5];
    let mut previous_index = 0;
    let mut name_next = false;
    let mut chunk_start = 0;
    let mut create_start = 0;
    for index in 0..tokens.len() {
        if tokens[index].token == Token::SemiColon {
            chunk_start = index + 1;
        }
        let keyword = match &tokens[index].token {
            Token::Whitespace(_) => continue,
            Token::Word(word) if word.quote_style.is_none() => word.keyword,
            _ => Keyword::NoKeyword,
        };
        if keyword == Keyword::CREATE {
            create_start = index;
        }
        if name_next {
            result.names.insert(tokens[index].span.start);
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
            // Recovery can anchor either the procedural chunk or its guarded CREATE action.
            // Save both original heads before RECURSIVE disappears from the parser tokens.
            if let Some(heads) = original_heads.as_deref_mut() {
                for start in [chunk_start, create_start] {
                    if let Some((location, head)) = super::statement_head::head(&tokens[start..]) {
                        heads.entry(location).or_insert(head);
                    }
                }
            }
            tokens[previous_index].token = Token::Whitespace(Whitespace::Space);
            name_next = true;
        }
        preceding.rotate_left(1);
        preceding[4] = keyword;
        previous_index = index;
    }
    result
}
