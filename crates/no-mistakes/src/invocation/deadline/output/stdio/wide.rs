use std::io;

/// Console APIs count UTF-16 units, including on a short write.
pub(super) fn write_all(
    text: &str,
    mut write: impl FnMut(&[u16]) -> io::Result<usize>,
) -> io::Result<()> {
    let units: Vec<_> = text.encode_utf16().collect();
    let mut remaining = units.as_slice();
    while !remaining.is_empty() {
        let mut length = remaining.len().min(8192);
        // A deliberate chunk boundary must not split a supplementary character.
        if length < remaining.len() && (0xd800..=0xdbff).contains(&remaining[length - 1]) {
            length -= 1;
        }
        let written = write(&remaining[..length])?;
        if written == 0 {
            return Err(io::ErrorKind::WriteZero.into());
        }
        if written > length {
            return Err(io::ErrorKind::InvalidData.into());
        }
        remaining = &remaining[written..];
    }
    Ok(())
}

#[cfg(test)]
#[path = "wide/tests.rs"]
mod tests;
