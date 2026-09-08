use super::EffectCallFact;
use crate::codebase::dependencies::extract::{FunctionCall, InvocationKind};
use std::collections::{HashMap, HashSet};

pub(crate) type EffectNames = HashMap<String, Option<String>>;

/// Projects configured effect occurrences from the canonical call collection.
///
/// Effects intentionally remain spelling-based: a configured terminal member
/// name still matches `client.createSubscriber()`, and a shadowed binding is
/// still reportable as an effect occurrence. Resolution-sensitive graph users
/// must inspect `FunctionCall::target_identity` instead.
pub(crate) fn collect_effect_calls(
    calls: &[FunctionCall],
    names: &EffectNames,
) -> Vec<EffectCallFact> {
    let scoped_calls: HashSet<_> = calls
        .iter()
        .filter(|call| {
            call.caller.is_some()
                && matches!(
                    call.invocation,
                    InvocationKind::Call | InvocationKind::Construct
                )
        })
        .map(|call| call.offset)
        .collect();
    calls
        .iter()
        .filter(|call| {
            // Exported variable initializers are also visited once while
            // resolving their value references. Keep that scoped occurrence;
            // the ordinary outer traversal records the same call with no
            // caller and is not a distinct effect.
            call.caller.is_some() || !scoped_calls.contains(&call.offset)
        })
        .filter(|call| {
            matches!(
                call.invocation,
                InvocationKind::Call | InvocationKind::Construct
            )
        })
        .filter_map(|call| {
            let (callee, category) = effect_match(&call.callee, names)?;
            Some(EffectCallFact {
                line: call.line as usize,
                callee: callee.to_string(),
                category: category.clone(),
                caller: call.syntactic_caller.clone(),
            })
        })
        .collect()
}

fn effect_match<'a>(
    callee: &'a str,
    names: &'a EffectNames,
) -> Option<(&'a str, &'a Option<String>)> {
    names
        .get_key_value(callee)
        .map(|(name, category)| (name.as_str(), category))
        .or_else(|| {
            callee
                .rsplit_once('.')
                .and_then(|(_, terminal)| names.get_key_value(terminal))
                .map(|(name, category)| (name.as_str(), category))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source;
    use oxc_allocator::Allocator;
    use oxc_span::SourceType;
    use std::path::Path;

    #[test]
    fn preserves_terminal_effects_on_non_identifier_receivers() {
        let source = "this.invalidate(); factory().invalidate();";
        let allocator = Allocator::default();
        let parsed = crate::ast::parse(
            Path::new("non-identifier-effects.ts"),
            &allocator,
            source,
            SourceType::ts(),
        );
        let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
        let effects = collect_effect_calls(
            &facts.function_calls,
            &EffectNames::from([("invalidate".to_string(), Some("cache".to_string()))]),
        );

        assert_eq!(effects.len(), 2);
        assert!(effects.iter().all(|effect| effect.callee == "invalidate"));
        assert!(effects
            .iter()
            .all(|effect| effect.category.as_deref() == Some("cache")));
    }
}
