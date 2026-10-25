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
        } else {
            if !super::constructor_aliases::unmutated_reference(
                file,
                &call.callee,
                call.offset,
                call.caller_id,
            ) {
                return;
            }
            let target = if call.target_identity == CallTargetIdentity::ModuleExport {
                self.reference(path, &call.callee)
                    .map(|target| (target, false))
            } else {
                self.aliased_constructor(path, file, call)
            };
            if let Some((target, nonnullish)) = target {
                self.output
                    .constructors
                    .entry(path.to_path_buf())
                    .or_default()
                    .insert(call.offset, target.clone());
                if let Some(site) = self
                    .constructor_sites
                    .get(path)
                    .and_then(|sites| sites.get(&call.offset))
                    .filter(|site| site.target_nodes().all(|node| node == &target))
                    .filter(|site| {
                        nonnullish
                            || matches!(
                                site.target,
                                crate::codebase::dependencies::graph::ResolvedCallTarget::Unknown
                            )
                    })
                {
                    self.output
                        .constructor_provenance
                        .entry(path.to_path_buf())
                        .or_default()
                        .insert(call.offset, (*site).clone());
                }
            }
        }
    }
}

fn global_constructor(file: &TsFileFacts, call: &FunctionCall) -> bool {
    // Bare constructors such as `Map`, `Date`, and `URL` are already
    // `CallTargetIdentity::Global` and resolve on the ordinary global path.
    // A dotted callee stays `Unknown`: the extractor marks only a bare name,
    // or a `globalThis` / `window` / `self` / `global` prefix, as `Global`.
    // `Intl` and `WebAssembly` are the standard globals whose constructors
    // are properties rather than bare identifiers, and none of them is an
    // `Error` root. Other dotted names can be packages or user objects, so
    // they stay unresolved. Do not add package or product names here.
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
