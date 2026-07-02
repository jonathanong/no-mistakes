pub(super) fn call(call: &str) -> String {
    let mut pieces = call.split('.');
    let first = pieces.next().unwrap_or_default();
    let rest: Vec<&str> = pieces.collect();
    if first.is_empty() || rest.is_empty() {
        return format!(
            r"{}(?P<call>{}{}\()",
            standalone_receiver_prefix(),
            regex::escape(call),
            type_args()
        );
    }
    let member = rest
        .into_iter()
        .map(|part| {
            let escaped = regex::escape(part);
            format!(
                r#"(?:\s*\.\s*{escaped}|\s*\?\.\s*{escaped}|\s*\[\s*['"]{escaped}['"]\s*\]|\s*\?\.\s*\[\s*['"]{escaped}['"]\s*\])"#
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r"{}(?P<call>{}{}{}\()",
        standalone_receiver_prefix(),
        regex::escape(first),
        member,
        type_args()
    )
}

pub(super) fn module(module: &str) -> String {
    let module = regex::escape(module);
    format!(
        r#"\bfrom\s+['"]{module}(?:['"/])|\bimport\s+['"]{module}(?:['"/])|\brequire\s*\(\s*['"`]{module}(?:['"`/])|\bimport\s*\(\s*['"`]{module}(?:['"`/])"#
    )
}

fn standalone_receiver_prefix() -> &'static str {
    r"(?:^|[^A-Za-z0-9_$.\]])"
}

fn type_args() -> &'static str {
    r"\s*(?:<[^;\n]*>)?\s*"
}
