use super::*;

/// Strip a `--- `/`+++ ` line's leading `a/`/`b/` prefix (forced by
/// `stream_git_diff`'s `--src-prefix=a/ --dst-prefix=b/`) and git's trailing
/// tab disambiguator, appended only when the path itself contains
/// whitespace (e.g. `--- a/a b/file.ts\t`), so it doesn't become part of the
/// path.
pub(super) fn strip_ab_prefix(raw: &Path) -> PathBuf {
    raw.strip_prefix("a")
        .or_else(|_| raw.strip_prefix("b"))
        .unwrap_or(raw)
        .to_path_buf()
}

pub(super) fn parse_hunk_header(line: &str) -> Option<DiffHunk> {
    // @@ -old_start,old_count +new_start,new_count @@ optional section
    let mut parts = line.split_whitespace();
    let old = parts.next()?;
    let old_range = parts.next()?;
    let new_range = parts.next()?;
    if old != "@@" || parts.next()? != "@@" {
        return None;
    }
    let (old_start, old_count) = parse_hunk_range(old_range.strip_prefix('-')?)?;
    let (new_start, new_count) = parse_hunk_range(new_range.strip_prefix('+')?)?;
    Some(DiffHunk {
        old_start,
        old_count,
        new_start,
        new_count,
        lines: Vec::new(),
    })
}

fn parse_hunk_range(range: &str) -> Option<(usize, usize)> {
    let (start, count) = range
        .split_once(',')
        .map_or((range, "1"), |(start, count)| (start, count));
    Some((start.parse().ok()?, count.parse().ok()?))
}

pub(super) fn parse_diff_header(line: &str) -> Result<(Option<PathBuf>, PathBuf)> {
    let rest = line.strip_prefix("diff --git ").unwrap_or("");
    if let Some(decoded) = parse_c_quoted_path(rest) {
        let (old_path, consumed) = decoded?;
        let remainder = rest[consumed..]
            .strip_prefix(' ')
            .context("quoted git diff header is missing its destination path")?;
        let new_path = parse_git_path(remainder)?;
        return Ok((Some(strip_ab_prefix(&old_path)), strip_ab_prefix(&new_path)));
    }
    if let Some((old_path, quoted_new_path)) = rest.split_once(" \"b/") {
        let new_path = parse_git_path(&format!("\"b/{quoted_new_path}"))?;
        return Ok((
            Some(strip_ab_prefix(Path::new(old_path))),
            strip_ab_prefix(&new_path),
        ));
    }
    // For anything but a rename, git shows the *same* path on both sides
    // (`a/P b/P`), so the correct split is wherever the two halves match —
    // not necessarily the first " b/" substring, which a path containing
    // that literal text (e.g. `a b/file.ts`) would misparse. This is the
    // only path source hunkless deletions/additions have (no `--- `/`+++ `
    // lines to fall back on), so it must be resolved here rather than left
    // to the `---`/`+++` preference in `PendingDiffFile::finish`.
    if let Some(path) = split_matching_ab_halves(rest) {
        return Ok((Some(PathBuf::from(path)), PathBuf::from(path)));
    }
    let (a_part, b_part) = match rest.split_once(" b/") {
        Some((a, b)) => (a.strip_prefix("a/").unwrap_or(a), b),
        None => return Ok((None, PathBuf::from(rest))),
    };
    Ok((Some(PathBuf::from(a_part)), PathBuf::from(b_part)))
}

pub(super) fn parse_git_path(raw: &str) -> Result<PathBuf> {
    let raw = raw.strip_suffix('\t').unwrap_or(raw);
    match parse_c_quoted_path(raw) {
        Some(decoded) => {
            let (path, consumed) = decoded?;
            if consumed != raw.len() {
                anyhow::bail!("quoted git path has trailing bytes")
            }
            Ok(path)
        }
        None => Ok(PathBuf::from(raw)),
    }
}

fn parse_c_quoted_path(raw: &str) -> Option<Result<(PathBuf, usize)>> {
    let bytes = raw.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                let path = match String::from_utf8(decoded) {
                    Ok(path) => path,
                    Err(error) => return Some(Err(error.into())),
                };
                return Some(Ok((PathBuf::from(path), index + 1)));
            }
            b'\\' => {
                index += 1;
                let Some(escaped) = bytes.get(index).copied() else {
                    return Some(Err(anyhow::anyhow!(
                        "quoted git path ends with an incomplete escape"
                    )));
                };
                match escaped {
                    b'a' => decoded.push(0x07),
                    b'b' => decoded.push(0x08),
                    b't' => decoded.push(b'\t'),
                    b'n' => decoded.push(b'\n'),
                    b'v' => decoded.push(0x0b),
                    b'f' => decoded.push(0x0c),
                    b'r' => decoded.push(b'\r'),
                    b'\\' | b'"' => decoded.push(escaped),
                    b'0'..=b'7' => {
                        let mut value = u16::from(escaped - b'0');
                        let mut digits = 1;
                        while digits < 3 {
                            let Some(next @ b'0'..=b'7') = bytes.get(index + 1).copied() else {
                                break;
                            };
                            value = value * 8 + u16::from(next - b'0');
                            index += 1;
                            digits += 1;
                        }
                        let value = match u8::try_from(value) {
                            Ok(value) => value,
                            Err(_) => {
                                return Some(Err(anyhow::anyhow!(
                                    "quoted git path contains an out-of-range octal escape"
                                )));
                            }
                        };
                        decoded.push(value);
                    }
                    _ => {
                        return Some(Err(anyhow::anyhow!(
                            "quoted git path contains an unsupported escape"
                        )));
                    }
                }
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    Some(Err(anyhow::anyhow!(
        "quoted git path is missing its closing quote"
    )))
}

/// Find the `" b/"` occurrence in a `diff --git a/X b/Y` header's `a/X b/Y`
/// remainder where stripping the leading `a/` from the left half yields a
/// string identical to the right half — i.e. `X == Y`, true for every
/// non-rename diff. Tries every occurrence (not just the first), so a path
/// containing the literal substring `" b/"` still resolves correctly. A
/// genuine rename (`X != Y`) never matches here and falls through to the
/// naive first-occurrence split, which is fine: a rename's real path comes
/// from `rename from`/`rename to` lines, not this header.
fn split_matching_ab_halves(rest: &str) -> Option<&str> {
    let a_part = rest.strip_prefix("a/")?;
    let mut search_from = 0;
    while let Some(offset) = a_part[search_from..].find(" b/") {
        let split_at = search_from + offset;
        let candidate = &a_part[..split_at];
        let remainder = &a_part[split_at + " b/".len()..];
        if candidate == remainder {
            return Some(candidate);
        }
        search_from = split_at + 1;
    }
    None
}
