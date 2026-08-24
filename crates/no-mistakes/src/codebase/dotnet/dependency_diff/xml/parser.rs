use super::super::DotnetDependencyDiagnostic;

pub(in crate::codebase::dotnet) fn parse_open_tag(
    tag: &str,
) -> Result<(String, Vec<(String, String)>, bool), DotnetDependencyDiagnostic> {
    let mut cursor = 0;
    let bytes = tag.as_bytes();
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    let name_start = cursor;
    while bytes
        .get(cursor)
        .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'/' | b'='))
    {
        cursor += 1;
    }
    if name_start == cursor {
        return Err(DotnetDependencyDiagnostic::MalformedXml);
    }
    let name = tag[name_start..cursor].to_string();
    let mut attributes = Vec::new();
    loop {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        match bytes.get(cursor) {
            None => return Ok((name, attributes, false)),
            Some(b'/') if bytes.get(cursor + 1).is_none() => return Ok((name, attributes, true)),
            Some(b'/') => return Err(DotnetDependencyDiagnostic::MalformedXml),
            _ => {}
        }
        let start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'/'))
        {
            cursor += 1;
        }
        if start == cursor {
            return Err(DotnetDependencyDiagnostic::MalformedXml);
        }
        let attribute = tag[start..cursor].to_string();
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            return Err(DotnetDependencyDiagnostic::MalformedXml);
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let quote = *bytes
            .get(cursor)
            .ok_or(DotnetDependencyDiagnostic::MalformedXml)?;
        if !matches!(quote, b'\'' | b'"') {
            return Err(DotnetDependencyDiagnostic::MalformedXml);
        }
        cursor += 1;
        let value_start = cursor;
        while bytes.get(cursor) != Some(&quote) {
            cursor += 1;
            if cursor >= bytes.len() {
                return Err(DotnetDependencyDiagnostic::MalformedXml);
            }
        }
        attributes.push((attribute, tag[value_start..cursor].to_string()));
        cursor += 1;
    }
}
