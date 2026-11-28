use super::{CallDelayArgs, DelayVerdict};
use crate::codebase::ts_source::module_facts::arguments::InstancePropertyValue;

pub(super) fn judge_instance_property(args: &CallDelayArgs<'_>, prop: &str) -> DelayVerdict {
    if args.values.is_none() || args.dynamic_scope {
        return DelayVerdict::Unanalyzable(
            "call arguments and instance properties could not be statically analyzed".to_string(),
        );
    }
    let Some(evidence) = args.instance else {
        return DelayVerdict::Missing(format!(
            "does not attach required `{prop}` delay property on the constructed instance"
        ));
    };
    if evidence.throw_count > 0 && !evidence.positive_through_throws.contains(prop) {
        return if evidence.opaque_mutation {
            DelayVerdict::Unanalyzable(format!(
                "constructed instance was thrown before `{prop}` was a proven positive delay"
            ))
        } else {
            DelayVerdict::Missing(format!(
                "threw the constructed instance before `{prop}` was a positive delay"
            ))
        };
    }
    if evidence.throw_count > 0 {
        return DelayVerdict::Valid;
    }
    match evidence.properties.get(prop) {
        Some(InstancePropertyValue::Positive) => DelayVerdict::Valid,
        Some(InstancePropertyValue::Invalid) => DelayVerdict::Missing(format!(
            "instance property `{prop}` must be a positive duration"
        )),
        Some(InstancePropertyValue::AbsentOrNonFinite) => DelayVerdict::Missing(format!(
            "instance property `{prop}` is undefined or non-finite"
        )),
        None if evidence.opaque_mutation => DelayVerdict::Unanalyzable(format!(
            "constructed instance escapes or is mutated opaquely, so `{prop}` could not be verified"
        )),
        None => DelayVerdict::Missing(format!(
            "does not attach required `{prop}` delay property on the constructed instance"
        )),
    }
}
