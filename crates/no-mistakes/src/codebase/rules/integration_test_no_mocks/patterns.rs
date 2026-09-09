pub(super) fn module(module: &str) -> String {
    let module = regex::escape(module);
    format!(
        r#"\bfrom\s+['"]{module}(?:['"/])|\bimport\s+['"]{module}(?:['"/])|\brequire\s*\(\s*['"\`]{module}(?:['"\`/])|\bimport\s*\(\s*['"\`]{module}(?:['"\`/])"#
    )
}
