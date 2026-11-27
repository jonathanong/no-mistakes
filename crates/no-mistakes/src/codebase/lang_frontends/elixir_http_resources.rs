use regex::Regex;
use std::sync::OnceLock;

pub(super) fn parse_resources(trimmed: &str) -> Option<Vec<(String, String)>> {
    let cap = resources_re().captures(trimmed)?;
    let path = cap.get(1)?.as_str();
    if path.contains("#{") {
        return None;
    }
    let controller = cap.get(2)?.as_str();
    Some(expand(path, controller))
}

fn expand(path: &str, controller: &str) -> Vec<(String, String)> {
    let base = path.trim_end_matches('/');
    let member = format!("{base}/:id");
    [
        ("index", base.to_string()),
        ("create", base.to_string()),
        ("new", format!("{base}/new")),
        ("show", member.clone()),
        ("edit", format!("{member}/edit")),
        ("update", member.clone()),
        ("delete", member),
    ]
    .into_iter()
    .map(|(action, path)| (path, format!("{controller}.{action}")))
    .collect()
}

fn resources_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"^resources\s+["']([^"']+)["']\s*,\s*([A-Z][A-Za-z0-9_.]*)\s*$"#)
            .expect("phoenix resources")
    })
}
