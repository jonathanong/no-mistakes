use super::KeyFormat;
use anyhow::{ensure, Context, Result};

pub(super) fn decode_input(format: KeyFormat, source: &[u8]) -> Result<std::borrow::Cow<'_, str>> {
    if matches!(format, KeyFormat::Strings | KeyFormat::Resx) {
        if let Some(bytes) = source.strip_prefix(&[0xff, 0xfe]) {
            return decode_utf16(bytes, u16::from_le_bytes).map(Into::into);
        }
        if let Some(bytes) = source.strip_prefix(&[0xfe, 0xff]) {
            return decode_utf16(bytes, u16::from_be_bytes).map(Into::into);
        }
    }
    let bytes = source.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(source);
    Ok(std::str::from_utf8(bytes)
        .context("expected UTF-8 text")?
        .into())
}

fn decode_utf16(bytes: &[u8], read: fn([u8; 2]) -> u16) -> Result<String> {
    let (pairs, remainder) = bytes.as_chunks::<2>();
    ensure!(remainder.is_empty(), "incomplete UTF-16 code unit");
    char::decode_utf16(pairs.iter().copied().map(read))
        .collect::<std::result::Result<_, _>>()
        .context("invalid UTF-16 text")
}
