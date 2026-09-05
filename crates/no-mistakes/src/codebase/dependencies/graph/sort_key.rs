/// Concatenated `node_sort_key` bytes.
///
/// Paths may contain `#`, so parts must not be compared sequentially.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct NodeSortKey {
    bytes: Box<[u8]>,
}

impl NodeSortKey {
    fn new(
        path: Option<Arc<Path>>,
        mid: &'static str,
        name: Option<Arc<str>>,
        step: Option<usize>,
    ) -> Self {
        // Match `node_sort_key` / `cmp_node_sort_keys`: valid UTF-8 is a
        // borrowed slice; invalid sequences use the same U+FFFD replacement
        // as `Path::to_string_lossy` so canonical ordinals stay stable.
        let path_lossy;
        let path_bytes: &[u8] = match path.as_ref() {
            None => b"",
            Some(path) => match path.to_str() {
                Some(s) => s.as_bytes(),
                None => {
                    path_lossy = path.to_string_lossy().into_owned();
                    path_lossy.as_bytes()
                }
            },
        };
        let mid_bytes = mid.as_bytes();
        let name_bytes = name.as_deref().map(str::as_bytes).unwrap_or(b"");
        let mut suffix = [0u8; 32];
        let step_bytes = match step {
            Some(step) => write_step_suffix(step, &mut suffix).as_bytes(),
            None => b"",
        };
        let mut bytes = Vec::with_capacity(
            path_bytes.len() + mid_bytes.len() + name_bytes.len() + step_bytes.len(),
        );
        bytes.extend_from_slice(path_bytes);
        bytes.extend_from_slice(mid_bytes);
        bytes.extend_from_slice(name_bytes);
        bytes.extend_from_slice(step_bytes);
        Self {
            bytes: bytes.into_boxed_slice(),
        }
    }
}

/// Byte-identical to concatenating the slices, then `memcmp`.
fn cmp_concatenated_bytes(left: &[&[u8]], right: &[&[u8]]) -> std::cmp::Ordering {
    let mut left_i = 0;
    let mut right_i = 0;
    let mut left_off = 0;
    let mut right_off = 0;
    loop {
        while left_i < left.len() && left_off >= left[left_i].len() {
            left_i += 1;
            left_off = 0;
        }
        while right_i < right.len() && right_off >= right[right_i].len() {
            right_i += 1;
            right_off = 0;
        }
        match (left_i < left.len(), right_i < right.len()) {
            (false, false) => return std::cmp::Ordering::Equal,
            (false, true) => return std::cmp::Ordering::Less,
            (true, false) => return std::cmp::Ordering::Greater,
            (true, true) => {
                let left_rest = &left[left_i][left_off..];
                let right_rest = &right[right_i][right_off..];
                let n = left_rest.len().min(right_rest.len());
                let order = left_rest[..n].cmp(&right_rest[..n]);
                if order != std::cmp::Ordering::Equal {
                    return order;
                }
                left_off += n;
                right_off += n;
            }
        }
    }
}
