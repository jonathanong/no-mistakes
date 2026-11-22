use super::{InventoryLockFact, Lexeme, Segment};
use sqlparser::ast::{LockType, NonBlock, Query, SetExpr, Spanned};

pub(super) fn collect(
    segment: &Segment<'_>,
    query: &Query,
    out: &mut Vec<InventoryLockFact>,
) -> Option<()> {
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
    let mut sites = Vec::new();
    while let Some(lock) = locks.next() {
        let (index, head) = heads.next()?;
        let tail = &segment.tokens[start + index + 1..];
        let (category, words) = strength(lock.lock_type, tail)?;
        sites.push((category, head));
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
            }
            tail = &tail[tail.partition_point(|token| token.at() < end)..];
        }
        if let Some(nonblock) = lock.nonblock {
            let (category, keyword) = match nonblock {
                NonBlock::Nowait => ("nowait", "nowait"),
                NonBlock::SkipLocked => ("skip-locked", "skip"),
            };
            sites.push((category, tail.first().filter(|token| token.word(keyword))?));
        }
    }
    for (category, token) in sites {
        segment.record(category, token, out);
    }
    Some(())
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

fn strength(kind: LockType, tokens: &[Lexeme<'_>]) -> Option<(&'static str, usize)> {
    match (kind, tokens) {
        (LockType::Update, [no, key, update, ..])
            if no.word("no") && key.word("key") && update.word("update") =>
        {
            Some(("for-no-key-update", 3))
        }
        (LockType::Update, [update, ..]) if update.word("update") => Some(("for-update", 1)),
        (LockType::Share, [key, share, ..]) if key.word("key") && share.word("share") => {
            Some(("for-key-share", 2))
        }
        (LockType::Share, [share, ..]) if share.word("share") => Some(("for-share", 1)),
        _ => None,
    }
}
