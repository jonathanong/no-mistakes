use std::io;
use std::path::Path;
use std::sync::Arc;

/// Read a UTF-8 source file into one `Arc<str>` without an intermediate
/// `String` copy after the filesystem buffer.
pub(super) fn read_utf8_arc(path: &Path) -> io::Result<Arc<str>> {
    let bytes = std::fs::read(path)?;
    let string = String::from_utf8(bytes).map_err(|error| {
        io::Error::new(io::ErrorKind::InvalidData, error.utf8_error())
    })?;
    Ok(Arc::from(string))
}
