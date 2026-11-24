use oxc_ast::{
    ast::{Argument, CallExpression, Expression, Program},
    AstKind,
};
use oxc_semantic::{Semantic, SemanticBuilder};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

mod trace;

#[derive(Debug, Clone, Default)]
pub struct FixtureReads {
    calls: BTreeMap<String, Option<ProvenReads>>,
}

#[derive(Debug, Clone, Default)]
struct ProvenReads {
    starts: BTreeSet<usize>,
    paths: BTreeSet<String>,
    argument: String,
    callee: Vec<String>,
}

impl FixtureReads {
    pub fn collect(path: &Path, source: &str, literals: &BTreeSet<String>) -> Self {
        crate::ast::with_program(path, source, |program, source| {
            Self::from_program(program, source, literals)
        })
        .unwrap_or_default()
    }

    pub fn from_program(program: &Program<'_>, source: &str, literals: &BTreeSet<String>) -> Self {
        let built = SemanticBuilder::new()
            .with_build_nodes(true)
            .with_check_syntax_error(true)
            .build(program);
        if !built.diagnostics.is_empty() {
            return Self::default();
        }
        Self::from_semantic(&built.semantic, source, literals)
    }

    pub fn from_semantic(
        semantic: &Semantic<'_>,
        source: &str,
        literals: &BTreeSet<String>,
    ) -> Self {
        let mut reads = Self::default();
        if semantic.scoping().root_scope_flags().contains_direct_eval() {
            return reads;
        }
        let mut arguments = BTreeMap::<_, usize>::new();
        let mut unknown_loaders = BTreeSet::new();
        for node in semantic.nodes().iter() {
            let call = match node.kind() {
                AstKind::WithStatement(_) => return Self::default(),
                AstKind::CallExpression(call) => call,
                _ => continue,
            };
            let callee = crate::ast::expression_path(&call.callee);
            let argument = match call.arguments.first() {
                Some(Argument::StringLiteral(argument)) => argument.value.as_str(),
                _ => None,
            };
            let Some(argument) = argument else {
                unknown_loaders.extend(callee);
                continue;
            };
            *arguments.entry(argument).or_default() += 1;
            if call.arguments.len() != 1 || !is_loader_candidate(call) {
                continue;
            }
            let Some(callee) = callee else {
                continue;
            };
            let start = call.span.start as usize;
            let Some(literal) = source.get(start..call.span.end as usize) else {
                continue;
            };
            if !literals.contains(literal) {
                continue;
            }
            let entry = reads.calls.entry(literal.to_owned()).or_insert_with(|| {
                Some(ProvenReads {
                    argument: argument.to_owned(),
                    callee,
                    ..ProvenReads::default()
                })
            });
            if let Some(proven) = entry {
                if let Some(paths) = trace::reads(semantic, node.id()) {
                    proven.starts.insert(start);
                    proven.paths.extend(paths);
                } else {
                    *entry = None;
                }
            }
        }
        // A second call with the same argument may load the fixture through another spelling.
        reads.calls.values_mut().for_each(|entry| {
            if entry.as_ref().is_some_and(|proven| {
                arguments.get(proven.argument.as_str()) != Some(&proven.starts.len())
                    || unknown_loaders.contains(&proven.callee)
            }) {
                *entry = None;
            }
        });
        reads
    }

    pub fn affects(&self, source: &str, literal: &str, changed: &[String]) -> bool {
        let Some(Some(reads)) = self.calls.get(literal) else {
            return true;
        };
        reads.paths.iter().any(|read| {
            changed
                .iter()
                .any(|path| contains_pointer(read, path) || contains_pointer(path, read))
        }) || !covers_every_occurrence(source, literal, &reads.starts)
    }
}

fn is_loader_candidate(call: &CallExpression<'_>) -> bool {
    if call.optional {
        return false;
    }
    let mut callee = &call.callee;
    while let Expression::StaticMemberExpression(member) = callee {
        if member.optional {
            return false;
        }
        callee = &member.object;
    }
    matches!(callee, Expression::Identifier(_))
}

fn contains_pointer(parent: &str, child: &str) -> bool {
    child
        .strip_prefix(parent)
        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'))
}

fn covers_every_occurrence(source: &str, literal: &str, starts: &BTreeSet<usize>) -> bool {
    let Some(first) = literal.chars().next() else {
        return false;
    };
    let mut offset = 0;
    while let Some(next) = source[offset..].find(literal) {
        let start = offset + next;
        if !starts.contains(&start) {
            return false;
        }
        offset = start + first.len_utf8();
    }
    true
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
