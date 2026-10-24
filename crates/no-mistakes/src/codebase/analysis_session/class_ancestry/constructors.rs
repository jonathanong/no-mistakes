use super::*;
use crate::codebase::dependencies::extract::FunctionCall;

impl Builder<'_> {
    pub(super) fn record_constructor(
        &mut self,
        path: &Path,
        file: &TsFileFacts,
        call: &FunctionCall,
        namespaces: &crate::codebase::dependencies::graph::NamespaceTable,
    ) {
        if call.invocation != InvocationKind::Construct || call.is_callback {
            return;
        }
        if namespaces.hides_constructor(call.caller_id, call.offset, &call.callee) {
            return;
        }
        if global_constructor(file, call) {
            self.output
                .global_constructors
                .entry(path.to_path_buf())
                .or_default()
                .insert(call.offset);
        } else if call.target_identity == CallTargetIdentity::ModuleExport {
            // Import bindings retain the prepared resolver/catalog semantics.
            // Only an actual class declaration supplies construction proof.
            if let Some(target) = self.reference(path, &call.callee) {
                self.output
                    .constructors
                    .entry(path.to_path_buf())
                    .or_default()
                    .insert(call.offset, target);
            }
        }
    }
}

fn global_constructor(file: &TsFileFacts, call: &FunctionCall) -> bool {
    // Standard intrinsic constructors, never package/product name exceptions.
    let name = ["globalThis.", "window.", "self.", "global."]
        .iter()
        .find_map(|prefix| call.callee.strip_prefix(prefix))
        .unwrap_or(&call.callee);
    let Some((root, member)) = name.split_once('.') else {
        return false;
    };
    let known = match root {
        "Intl" => [
            "Collator",
            "DateTimeFormat",
            "DisplayNames",
            "DurationFormat",
            "ListFormat",
            "Locale",
            "NumberFormat",
            "PluralRules",
            "RelativeTimeFormat",
            "Segmenter",
        ]
        .contains(&member),
        "WebAssembly" => [
            "Module",
            "Instance",
            "Memory",
            "Table",
            "Global",
            "Tag",
            "Exception",
        ]
        .contains(&member),
        _ => false,
    };
    if !known {
        return false;
    }
    call.target_identity == CallTargetIdentity::Global
        || (call.target_identity == CallTargetIdentity::Unknown
            && call
                .callee
                .split_once('.')
                .is_some_and(|(head, _)| head == root)
            && call.callee_binding_scope.is_none()
            && !file
                .imported_bindings
                .iter()
                .any(|binding| binding.local == root))
}
