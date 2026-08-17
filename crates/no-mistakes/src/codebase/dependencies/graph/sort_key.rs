/// Interned sort parts. Compares as the concatenation of `node_sort_key`.
#[derive(Clone, Debug)]
struct NodeSortKey {
    path: Option<Arc<Path>>,
    mid: &'static str,
    name: Option<Arc<str>>,
    step: [u8; 32],
    step_len: u8,
}

impl NodeSortKey {
    fn new(
        path: Option<Arc<Path>>,
        mid: &'static str,
        name: Option<Arc<str>>,
        step: Option<usize>,
    ) -> Self {
        let mut suffix = [0u8; 32];
        let step_len = match step {
            Some(step) => {
                let written = write_step_suffix(step, &mut suffix).len();
                debug_assert!(written <= suffix.len());
                debug_assert!(written <= usize::from(u8::MAX));
                written as u8
            }
            None => 0,
        };
        Self {
            path,
            mid,
            name,
            step: suffix,
            step_len,
        }
    }

    fn parts(&self) -> [&[u8]; 4] {
        [
            self.path
                .as_ref()
                .map(|path| path.as_os_str().as_encoded_bytes())
                .unwrap_or(b""),
            self.mid.as_bytes(),
            self.name.as_deref().map(str::as_bytes).unwrap_or(b""),
            &self.step[..usize::from(self.step_len)],
        ]
    }
}

impl PartialEq for NodeSortKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for NodeSortKey {}

impl PartialOrd for NodeSortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeSortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        cmp_concatenated_bytes(&self.parts(), &other.parts())
    }
}

fn cmp_concatenated_bytes(left: &[&[u8]], right: &[&[u8]]) -> std::cmp::Ordering {
    left.iter()
        .copied()
        .flatten()
        .cmp(right.iter().copied().flatten())
}
