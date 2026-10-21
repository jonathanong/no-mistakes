use super::*;
// Bound simultaneous metadata snapshots and retained prepared private suffixes.
pub(super) const MAX_FACTORY_ARMS: usize = 8;
pub(super) fn eligible<F: Fn(&str, &Path) -> Option<PathBuf>>(
    evaluator: &Evaluator<'_, F>,
    arms: &[Expr],
    path: &Path,
    env: Environment,
    depth: u8,
) -> bool {
    if !(2..=MAX_FACTORY_ARMS).contains(&arms.len())
        || depth < 8
        || evaluator.files[path].facts.precise_records
        || evaluator.mapped_arguments.contains_key(&env)
        || !evaluator.active_module_initials.is_empty()
    {
        return false;
    }
    arms.iter().all(|arm| {
        let Expr::Call {
            callee,
            args,
            start,
            end,
        } = arm
        else {
            return false;
        };
        let Expr::Name(name) = callee.as_ref() else {
            return false;
        };
        if name == "require"
            || evaluator.files[path].executors.contains(&(*start, *end))
            || !args.iter().all(|arg| {
                matches!(
                    arg,
                    Expr::Text(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Primitive
                )
            })
        {
            return false;
        }
        matches!(evaluator.scopes[env].get(name), Some(Value::Function(function, source, captured))
            if source == path && *captured == env && function.closed_factory
                && function.supported && !function.arrow && !function.asynchronous)
    })
}
