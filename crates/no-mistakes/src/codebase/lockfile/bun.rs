use super::{ResolutionKind, ResolvedPackage};

pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    let stripped = strip_jsonc(content);
    let Ok(root) = serde_json::from_str::<serde_json::Value>(&stripped) else {
        return Vec::new();
    };
    let Some(packages) = root.get("packages").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    packages
        .iter()
        .filter_map(|(name, entry)| {
            let arr = entry.as_array()?;
            let specifier = arr.first().and_then(|v| v.as_str()).unwrap_or("");
            let version = specifier.rsplit_once('@').map(|(_, v)| v).unwrap_or("");
            let info = arr.get(2);
            let integrity = info
                .and_then(|v| v.get("integrity"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = info
                .and_then(|v| v.get("resolved"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let fingerprint = if integrity.is_empty() {
                resolved.to_string()
            } else {
                integrity.to_string()
            };
            Some(ResolvedPackage {
                name: name.clone(),
                version: version.to_string(),
                fingerprint,
                kind: ResolutionKind::Registry,
            })
        })
        .collect()
}

// Strip JSONC syntax (// and /* */ comments, trailing commas) so serde_json can parse it.
fn strip_jsonc(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_str = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' && i + 1 < bytes.len() {
                out.push(b);
                out.push(bytes[i + 1]);
                i += 2;
            } else {
                if b == b'"' {
                    in_str = false;
                }
                out.push(b);
                i += 1;
            }
            continue;
        }
        if b == b'"' {
            in_str = true;
            out.push(b);
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    if bytes[i] == b'\n' {
                        out.push(b'\n');
                    }
                    i += 1;
                }
                i += 2;
                continue;
            }
        }
        if b == b',' {
            let mut j = i + 1;
            while j < bytes.len()
                && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\r' || bytes[j] == b'\n')
            {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                i += 1;
                continue;
            }
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_default()
}

#[cfg(test)]
mod tests;
