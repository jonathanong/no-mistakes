use super::*;
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn environment(
        &mut self,
        values: FxHashMap<String, Value>,
    ) -> Environment {
        let env = self.scopes.allocate(values.into());
        self.register_frame_references(env);
        env
    }
    pub(super) fn expr(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        let Some(depth) = depth.checked_sub(1) else {
            return Value::Failure("evaluation-depth-limit");
        };
        let value = if let Some(value) = self.atomic(expr, path, env, (depth, generic)) {
            value
        } else {
            match expr {
                Expr::Binary(op, left, right) => {
                    self.binary(op, left, right, path, env, (depth, generic))
                }
                Expr::Flow(steps) => self.steps(steps, path, &mut env.clone(), depth, generic),
                Expr::Template(parts) => self.template(parts, path, env, (depth, generic)),
                Expr::Append(base, tail) => self.append(base, tail, path, env, (depth, generic)),
                Expr::Tagged(tag, parts, effects) => {
                    self.tagged(tag, parts, effects, path, env, (depth, generic))
                }
                Expr::Sequence(parts) => self.sequence(parts, path, env, (depth, generic)),
                Expr::Discard(expr) => self.discard(expr, path, env, (depth, generic)),
                Expr::Alternatives(arms) => self.alternatives(arms, path, env, depth, generic),
                Expr::Spread(expr) => {
                    Value::Aggregate(vec![self.expr(expr, path, env, depth, generic)].into())
                }
                Expr::OpaqueCallback(expr) => {
                    match self.expr(expr, path, env, depth, generic).exposed() {
                        Value::Function(mut function, path, captured) => {
                            Arc::make_mut(&mut function).supported = false;
                            Value::Function(function, path, captured)
                        }
                        _ => Value::Unknown,
                    }
                }
                Expr::Await(expr) => self.await_expr(expr, path, env, depth, generic),
                Expr::Call {
                    callee,
                    args,
                    start,
                    end,
                } => self.call(callee, args, (*start, *end), path, env, (depth, generic)),
                Expr::ModuleLoad(specifier) => self.load_dynamic_import(specifier, path),
                Expr::Delete(children, index) => {
                    self.deleted(children, *index, path, env, (depth, generic))
                }
                Expr::Opaque(children) => self.opaque(children, &[], path, env, (depth, generic)),
                Expr::OptionalCall { call, children } => {
                    self.optional_call(call, children, path, env, (depth, generic))
                }
                Expr::OpaqueWrite { children, targets } => {
                    self.opaque(children, targets, path, env, (depth, generic))
                }
                Expr::SlotWrite {
                    receiver,
                    index,
                    value,
                } => self.slot_write(receiver, *index, value, path, env, (depth, generic)),
                Expr::Member(object, name) => {
                    self.member_expr(object, name, path, env, (depth, generic))
                }
                Expr::ComputedMember(object, key) => {
                    let value = self.operand(object, path, env, (depth, generic)).exposed();
                    self.expr(key, path, env, depth, generic);
                    self.member(value, "", path)
                }
                Expr::Index(object, index) => {
                    let value = self.expr(object, path, env, depth, generic).exposed();
                    self.index(value, *index)
                }
                Expr::Children(children)
                | Expr::Array(children)
                | Expr::Container(children)
                | Expr::Object(children)
                | Expr::Record(children, _) => {
                    self.container(expr, children, path, env, (depth, generic))
                }
                Expr::Unknown
                | Expr::Primitive
                | Expr::Number(_)
                | Expr::Boolean(_)
                | Expr::Unsupported
                | Expr::Text(_)
                | Expr::Name(_)
                | Expr::Function(_) => unreachable!("atomic expression"),
            }
        };
        if matches!(&value, Value::Prefix(_, _, Some(id)) if self.invalidated_builders.contains(id))
        {
            if self.files[path].facts.precise_records {
                Value::Failure("opaque-builder-effect")
            } else {
                Value::Unknown
            }
        } else {
            value
        }
    }
}
