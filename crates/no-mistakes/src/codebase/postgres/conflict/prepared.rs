//! Project conflict metadata from the AST owned by the shared statement pass.
use super::{collect::collect_statement, raw::raw_conflicts, SqlConflictInsertFact};
use sqlparser::ast::Statement;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::sync::Arc;

pub(in crate::codebase::postgres) fn analyze_parsed(
    sql: &str,
    statements: &[Statement],
    binds: &[(u32, u32)],
) -> anyhow::Result<Vec<SqlConflictInsertFact>> {
    let mut raw = raw_conflicts(sql)?.into_iter();
    let mut inserts = Vec::new();
    for statement in statements {
        let mut executed = Vec::new();
        crate::codebase::postgres::statements::walk_executed(statement, &mut executed);
        for statement in executed {
            collect_statement(statement, &mut raw, binds, &mut inserts)?;
        }
    }
    anyhow::ensure!(
        raw.next().is_none(),
        "could not align ON CONFLICT clauses with INSERT statements"
    );
    Ok(inserts)
}

pub(in crate::codebase::postgres) struct ConflictRecovery {
    pub reliable: Vec<SqlConflictInsertFact>,
    pub failure_overlaps_conflict: bool,
}

/// Per-chunk inputs for a parse that already failed somewhere in the variant.
pub(in crate::codebase::postgres) struct ConflictChunks<'a> {
    pub complete: &'a [bool],
    pub chunks: &'a [Option<Arc<[TokenWithSpan]>>],
    pub orphans: &'a [Arc<[TokenWithSpan]>],
}

impl ConflictChunks<'static> {
    pub(in crate::codebase::postgres) const EMPTY: Self = Self {
        complete: &[],
        chunks: &[],
        orphans: &[],
    };
}

/// Keep conflicts whose own semicolon chunk parsed completely.
/// Raw targets are read from that chunk, not from the surrounding variant.
pub(in crate::codebase::postgres) fn reliable_conflicts(
    statements: &[Statement],
    binds: &[(u32, u32)],
    complete: &[bool],
    chunks: &[Option<&str>],
    orphans: &[&str],
) -> ConflictRecovery {
    let mut recovery = ConflictRecovery {
        reliable: Vec::new(),
        failure_overlaps_conflict: false,
    };
    let mut index = 0;
    while index < statements.len() {
        let text = chunks.get(index).copied().flatten();
        let mut end = index + 1;
        if let Some(text) = text {
            while end < statements.len()
                && chunks
                    .get(end)
                    .and_then(|chunk| chunk.as_ref())
                    .is_some_and(|next| std::ptr::eq(*next, text))
            {
                end += 1;
            }
        }
        recover_chunk(
            text,
            &statements[index..end],
            complete.get(index..end).unwrap_or(&[]),
            binds,
            &mut recovery,
        );
        index = end;
    }
    for text in orphans {
        recover_chunk(Some(text), &[], &[], binds, &mut recovery);
    }
    recovery
}

pub(in crate::codebase::postgres) fn reliable_chunk_tokens(
    statements: &[Statement],
    binds: &[(u32, u32)],
    chunks: &ConflictChunks<'_>,
) -> ConflictRecovery {
    let mut texts = Vec::new();
    let mut ids = Vec::new();
    let mut previous: Option<(usize, usize)> = None;
    for chunk in chunks.chunks {
        let Some(tokens) = chunk else {
            ids.push(None);
            previous = None;
            continue;
        };
        let ptr = Arc::as_ptr(tokens) as *const () as usize;
        if let Some((_, id)) = previous.filter(|(previous_ptr, _)| *previous_ptr == ptr) {
            ids.push(Some(id));
            continue;
        }
        texts.push(render_chunk(tokens));
        let id = texts.len() - 1;
        ids.push(Some(id));
        previous = Some((ptr, id));
    }
    let rendered = ids
        .iter()
        .map(|id| id.map(|id| texts[id].as_str()))
        .collect::<Vec<_>>();
    let orphan_text = chunks
        .orphans
        .iter()
        .map(|tokens| render_chunk(tokens))
        .collect::<Vec<_>>();
    let orphans = orphan_text.iter().map(String::as_str).collect::<Vec<_>>();
    reliable_conflicts(statements, binds, chunks.complete, &rendered, &orphans)
}

fn render_chunk(tokens: &[TokenWithSpan]) -> String {
    let mut text = String::new();
    for token in tokens {
        if matches!(token.token, Token::EOF) {
            continue;
        }
        use std::fmt::Write;
        let _ = write!(text, "{token}");
    }
    text
}

/// Missing chunk text does not fall back to the whole variant. An incomplete
/// chunk with no `ON CONFLICT`, such as `VACUUM (`, does not overlap.
fn recover_chunk(
    text: Option<&str>,
    statements: &[Statement],
    complete: &[bool],
    binds: &[(u32, u32)],
    recovery: &mut ConflictRecovery,
) {
    let Some(text) = text else {
        if statements
            .iter()
            .enumerate()
            .any(|(index, _)| !complete.get(index).copied().unwrap_or(false))
        {
            recovery.failure_overlaps_conflict = true;
        }
        return;
    };
    let Ok(raw) = raw_conflicts(text) else {
        recovery.failure_overlaps_conflict = true;
        return;
    };
    let mut raw = raw.into_iter();
    let mut local = Vec::new();
    let mut failed = false;
    for statement in statements {
        if failed {
            break;
        }
        let mut executed = Vec::new();
        crate::codebase::postgres::statements::walk_executed(statement, &mut executed);
        for statement in executed {
            if collect_statement(statement, &mut raw, binds, &mut local).is_err() {
                failed = true;
                break;
            }
        }
    }
    let leftover = raw.next().is_some();
    let all_complete = !statements.is_empty()
        && statements
            .iter()
            .enumerate()
            .all(|(index, _)| complete.get(index).copied().unwrap_or(false));
    if all_complete && !failed && !leftover {
        recovery.reliable.extend(local);
    } else if failed || !local.is_empty() || leftover {
        recovery.failure_overlaps_conflict = true;
    }
}
