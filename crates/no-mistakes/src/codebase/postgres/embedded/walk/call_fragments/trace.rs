use super::MAX_PREFIX_BYTES;
use crate::codebase::postgres::embedded::{CallSqlFragment, MAX_EMBEDDED_SQL_VARIANTS};

#[derive(Clone, Default)]
pub(super) struct BuilderTrace {
    pub(super) prefix: Option<String>,
    pub(super) alt_prefixes: Vec<String>,
    pub(super) fragments: Vec<CallSqlFragment>,
    pub(super) decl_depth: (usize, usize),
}

impl BuilderTrace {
    pub(super) fn from_fragments(fragments: Vec<CallSqlFragment>) -> Self {
        if fragments.iter().any(|item| item.append_site.is_some()) {
            let mut composed = String::new();
            for item in &fragments {
                if composed.len() + item.sql.len() <= MAX_PREFIX_BYTES {
                    composed.push_str(&item.sql);
                }
            }
            return Self {
                prefix: (!composed.is_empty()).then_some(composed),
                alt_prefixes: Vec::new(),
                fragments,
                decl_depth: (0, 0),
            };
        }
        let (prefix, alt_prefixes) = if fragments.len() == 1 {
            let p = (fragments[0].sql.len() <= MAX_PREFIX_BYTES).then(|| fragments[0].sql.clone());
            (p, Vec::new())
        } else {
            let alts = fragments
                .iter()
                .filter(|item| item.sql.len() <= MAX_PREFIX_BYTES)
                .map(|item| item.sql.clone())
                .collect();
            (None, alts)
        };
        Self {
            prefix,
            alt_prefixes,
            fragments,
            decl_depth: (0, 0),
        }
    }

    pub(super) fn merge(&mut self, other: Self) {
        if self.fragments.is_empty() && self.prefix.is_none() && self.alt_prefixes.is_empty() {
            *self = other;
            return;
        }
        let mut prefixes = Vec::new();
        for p in self
            .prefix
            .take()
            .into_iter()
            .chain(self.alt_prefixes.drain(..))
            .chain(other.prefix)
            .chain(other.alt_prefixes)
        {
            if !prefixes.contains(&p) && prefixes.len() < MAX_EMBEDDED_SQL_VARIANTS {
                prefixes.push(p);
            }
        }
        if prefixes.len() == 1 {
            self.prefix = prefixes.pop();
        } else {
            self.alt_prefixes = prefixes;
        }
        self.fragments.extend(other.fragments);
    }
}

pub(super) fn extend_prefix(
    trace: &mut BuilderTrace,
    added: &[CallSqlFragment],
    straight_line: bool,
) {
    if !straight_line || added.is_empty() {
        return;
    }
    if trace.alt_prefixes.is_empty() {
        if added.len() == 1 {
            if let Some(p) = trace
                .prefix
                .as_mut()
                .filter(|p| p.len() + added[0].sql.len() <= MAX_PREFIX_BYTES)
            {
                p.push_str(&added[0].sql);
            }
        } else if added.len() <= MAX_EMBEDDED_SQL_VARIANTS {
            let base = trace.prefix.take().unwrap_or_default();
            trace.alt_prefixes = added
                .iter()
                .filter(|f| base.len() + f.sql.len() <= MAX_PREFIX_BYTES)
                .map(|f| format!("{base}{}", f.sql))
                .collect();
        }
        return;
    }
    let mut next = Vec::new();
    for p in &trace.alt_prefixes {
        let mut matched = false;
        for frag in added
            .iter()
            .filter(|f| f.context_sql.as_deref() == Some(p.as_str()))
        {
            matched = true;
            if p.len() + frag.sql.len() <= MAX_PREFIX_BYTES
                && next.len() < MAX_EMBEDDED_SQL_VARIANTS
            {
                let candidate = format!("{p}{}", frag.sql);
                if !next.contains(&candidate) {
                    next.push(candidate);
                }
            }
        }
        if !matched && next.len() < MAX_EMBEDDED_SQL_VARIANTS && !next.contains(p) {
            next.push(p.clone());
        }
    }
    if !next.is_empty() {
        trace.alt_prefixes = next;
    }
}
