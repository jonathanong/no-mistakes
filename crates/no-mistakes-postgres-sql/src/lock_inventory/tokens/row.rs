use super::{InventoryLockFact, Lexeme, Segment};
use sqlparser::ast::{LockType, NonBlock, Query, SetExpr, Spanned};

pub(super) fn collect(
    segment: &Segment<'_>,
    query: &Query,
    out: &mut Vec<InventoryLockFact>,
) -> Option<()> {
    for (mode, head, wait, _) in clauses(segment, query)? {
        segment.record(mode.category(), head, out);
        if let Some((category, token)) = wait {
            segment.record(category, token, out);
        }
    }
    Some(())
}

pub(super) fn modes(segment: &Segment<'_>, query: &Query) -> Option<Vec<RowLockMode>> {
    Some(
        clauses(segment, query)?
            .into_iter()
            .flat_map(|(mode, _, _, count)| std::iter::repeat_n(mode, count))
            .collect(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RowLockMode {
    KeyShare,
    Share,
    NoKeyUpdate,
    Update,
}

impl RowLockMode {
    fn category(self) -> &'static str {
        match self {
            Self::KeyShare => "for-key-share",
            Self::Share => "for-share",
            Self::NoKeyUpdate => "for-no-key-update",
            Self::Update => "for-update",
        }
    }
}

type Wait<'a, 'sql> = Option<(&'static str, &'a Lexeme<'sql>)>;
type Clause<'a, 'sql> = (RowLockMode, &'a Lexeme<'sql>, Wait<'a, 'sql>, usize);

fn clauses<'a, 'sql>(segment: &'a Segment<'sql>, query: &Query) -> Option<Vec<Clause<'a, 'sql>>> {
    let span = query.span();
    let head = segment.tokens.get(segment.index_at(span.start))?;
    let parentheses = if query.with.is_some() {
        0
    } else {
        leading_parentheses(&query.body)
    };
    let depth = head.depth.saturating_sub(parentheses);
    // The AST body excludes the lock clauses, including when LIMIT follows them.
    let start = segment.index_at(query.body.span().end);
    let mut heads = segment.tokens[start..]
        .iter()
        .enumerate()
        .take_while(|(_, token)| token.depth >= depth)
        .filter(|(_, token)| token.depth == depth && token.word("for"))
        .peekable();
    let mut locks = query.locks.iter().peekable();
    let mut clauses = Vec::new();
    while let Some(lock) = locks.next() {
        let (index, head) = heads.next()?;
        let tail = &segment.tokens[start + index + 1..];
        let (mode, words) = strength(lock.lock_type, tail)?;
        let mut count = 1;
        let mut tail = &tail[words..];
        if let Some(target) = &lock.of {
            // Expanded OF targets retain their original spans within one physical clause.
            let limit = heads.peek().map_or(segment.end(), |(_, next)| next.at());
            let mut end = target.span().end;
            while let Some(next) = locks
                .peek()
                .and_then(|lock| lock.of.as_ref())
                .filter(|name| name.span().start < limit)
            {
                end = next.span().end;
                locks.next();
                count += 1;
            }
            tail = &tail[tail.partition_point(|token| token.at() < end)..];
        }
        let wait = if let Some(nonblock) = lock.nonblock {
            let (category, keyword) = match nonblock {
                NonBlock::Nowait => ("nowait", "nowait"),
                NonBlock::SkipLocked => ("skip-locked", "skip"),
            };
            Some((category, tail.first().filter(|token| token.word(keyword))?))
        } else {
            None
        };
        clauses.push((mode, head, wait, count));
    }
    Some(clauses)
}

fn leading_parentheses(body: &SetExpr) -> usize {
    match body {
        SetExpr::Query(query) => {
            1 + if query.with.is_none() {
                leading_parentheses(&query.body)
            } else {
                0
            }
        }
        SetExpr::SetOperation { left, .. } => leading_parentheses(left),
        _ => 0,
    }
}

fn strength(kind: LockType, tokens: &[Lexeme<'_>]) -> Option<(RowLockMode, usize)> {
    match (kind, tokens) {
        (LockType::Update, [no, key, update, ..])
            if no.word("no") && key.word("key") && update.word("update") =>
        {
            Some((RowLockMode::NoKeyUpdate, 3))
        }
        (LockType::Update, [update, ..]) if update.word("update") => Some((RowLockMode::Update, 1)),
        (LockType::Share, [key, share, ..]) if key.word("key") && share.word("share") => {
            Some((RowLockMode::KeyShare, 2))
        }
        (LockType::Share, [share, ..]) if share.word("share") => Some((RowLockMode::Share, 1)),
        _ => None,
    }
}
