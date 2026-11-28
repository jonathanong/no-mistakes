use crate::codebase::dependencies::extract::{
    CallTargetIdentity, FunctionCall, ImportedBindingKind, InvocationKind,
};
use crate::codebase::ts_source::facts::TsFileFacts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VitestReceiver<'a> {
    Direct,
    Unproven { imported_from: Option<&'a str> },
}

pub(crate) struct VitestImportCall<'a> {
    pub(crate) line: u32,
    pub(crate) specifier: &'a str,
    pub(crate) computed: bool,
    pub(crate) receiver: VitestReceiver<'a>,
}

pub(crate) fn vitest_import_calls(
    facts: &TsFileFacts,
) -> impl Iterator<Item = VitestImportCall<'_>> {
    facts.function_calls.iter().filter_map(|call| {
        let receiver = vitest_receiver(call, facts)?;
        Some(VitestImportCall {
            line: call.line,
            specifier: call.static_arg.as_deref().unwrap_or("<computed>"),
            computed: call.static_arg.is_none(),
            receiver,
        })
    })
}

fn vitest_receiver<'a>(call: &FunctionCall, facts: &'a TsFileFacts) -> Option<VitestReceiver<'a>> {
    let (receiver, method) = call.callee.rsplit_once('.')?;
    if call.invocation != InvocationKind::Call
        || !matches!(method, "importActual" | "importMock")
        || call.target_identity == CallTargetIdentity::RepositoryFunction
    {
        return None;
    }
    let (head, member) = receiver
        .split_once('.')
        .map_or((receiver, None), |(head, member)| (head, Some(member)));
    let binding = if call.callee_binding_scope.is_none() {
        facts
            .imported_bindings
            .iter()
            .find(|binding| !binding.is_type_only && binding.local == head)
    } else {
        None
    };
    if let Some(binding) = binding {
        if binding.specifier == "vitest" {
            let direct = match member {
                Some("vi" | "vitest") => binding.kind == ImportedBindingKind::Namespace,
                None => {
                    binding.kind == ImportedBindingKind::Named
                        && matches!(binding.imported.as_str(), "vi" | "vitest")
                }
                _ => false,
            };
            return direct.then_some(VitestReceiver::Direct);
        }
        return Some(VitestReceiver::Unproven {
            imported_from: Some(binding.specifier.as_str()),
        });
    }
    if call.callee_binding_scope.is_none() && receiver == "vi" {
        return Some(VitestReceiver::Direct);
    }
    Some(VitestReceiver::Unproven {
        imported_from: None,
    })
}
