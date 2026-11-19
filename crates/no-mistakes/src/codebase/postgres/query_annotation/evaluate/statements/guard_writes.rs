use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    /// An imprecise guard test is not evaluated, so every binding it writes
    /// stops describing its annotation.
    pub(super) fn poison_guard_writes(&mut self, test: &Expr, env: Environment) {
        let mut targets = Vec::new();
        written_names(test, &mut targets);
        for name in targets {
            self.write_captured_binding(env, name, &Value::Unsupported);
        }
    }
}

fn written_names<'a>(expr: &'a Expr, out: &mut Vec<&'a str>) {
    match expr {
        Expr::OpaqueWrite { children, targets } => {
            out.extend(targets.iter().map(String::as_str));
            children.iter().for_each(|child| written_names(child, out));
        }
        Expr::SlotWrite {
            receiver, value, ..
        } => {
            if let Expr::Name(name) = receiver.as_ref() {
                out.push(name);
            }
            written_names(receiver, out);
            written_names(value, out);
        }
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => written_names(inner, out),
        Expr::Binary(_, left, right)
        | Expr::ComputedMember(left, right)
        | Expr::Append(left, right) => {
            written_names(left, out);
            written_names(right, out);
        }
        Expr::Call { callee, args, .. } => {
            written_names(callee, out);
            args.iter().for_each(|arg| written_names(arg, out));
        }
        Expr::Tagged(_, parts, effects) => parts
            .iter()
            .chain(effects)
            .for_each(|part| written_names(part, out)),
        Expr::Template(parts)
        | Expr::Children(parts)
        | Expr::Array(parts)
        | Expr::Container(parts)
        | Expr::Object(parts)
        | Expr::Record(parts, _)
        | Expr::Sequence(parts)
        | Expr::Alternatives(parts)
        | Expr::Opaque(parts)
        | Expr::OptionalCall {
            children: parts, ..
        }
        | Expr::Delete(parts, _) => parts.iter().for_each(|part| written_names(part, out)),
        Expr::Flow(_)
        | Expr::ModuleLoad(_)
        | Expr::Boolean(_)
        | Expr::Number(_)
        | Expr::Primitive
        | Expr::Unknown
        | Expr::Unsupported
        | Expr::Text(_)
        | Expr::Name(_)
        | Expr::Function(_) => {}
    }
}
