use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn readonly_alternatives(
        &mut self,
        arms: &[Expr],
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Option<Value> {
        // Local reads retain existing identities and captured frames. Prove
        // every arm before evaluation so namespace loads cannot slip through.
        // expr() rejects every depth-zero arm before evaluating any syntax.
        // Keep the recursive proof strict: a parent member may load namespaces.
        if depth != 0
            && !arms
                .iter()
                .all(|arm| self.readonly_arm(arm, path, *env, depth))
        {
            return None;
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.readonly_alternatives", 1);
            if depth == 0 {
                observer.increment("postgres.annotation.depth_limited_alternatives", 1);
            }
        }
        let returned = arms
            .iter()
            .map(|arm| self.expr(arm, path, env, depth, generic))
            .collect();
        Some(super::super::concat::alternative_result(
            returned,
            self.files[path].facts.precise_records,
        ))
    }

    fn readonly_arm(&self, arm: &Expr, path: &Path, env: Environment, depth: u8) -> bool {
        // Match expr's depth bound, including while proving nested reads. Do
        // not replace this with an unbounded recursive syntax classification.
        let Some(depth) = depth.checked_sub(1) else {
            return false;
        };
        match arm {
            Expr::Unknown
            | Expr::Primitive
            | Expr::Unsupported
            | Expr::Number(_)
            | Expr::Boolean(_)
            | Expr::Text(_) => true,
            Expr::Name(name) => {
                !self.mapped_parameter_unknown(env, name) && self.scopes[env].contains_key(name)
            }
            // Constructors evaluate children through operand(), which may
            // preserve effects even when the child itself only reads a value.
            Expr::Record(parts, _)
            | Expr::Object(parts)
            | Expr::Container(parts)
            | Expr::Children(parts) => parts.iter().all(|part| {
                self.readonly_arm(part, path, env, depth)
                    && !self.effect_can_mutate(part, path, env)
            }),
            Expr::Member(object, _) | Expr::Index(object, _) => {
                self.readonly_arm(object, path, env, depth)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
