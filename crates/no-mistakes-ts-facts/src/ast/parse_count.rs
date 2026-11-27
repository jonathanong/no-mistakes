use std::path::{Path, PathBuf};

struct ParseCountSession {
    owner: std::thread::ThreadId,
    owner_only: bool,
    counts: std::collections::HashMap<PathBuf, usize>,
}

type ParseCounts = std::collections::HashMap<PathBuf, ParseCountSession>;

fn parse_counts() -> &'static std::sync::Mutex<ParseCounts> {
    static COUNTS: std::sync::OnceLock<std::sync::Mutex<ParseCounts>> = std::sync::OnceLock::new();
    COUNTS.get_or_init(|| std::sync::Mutex::new(ParseCounts::new()))
}

#[doc(hidden)]
pub fn begin_parse_count(root: &Path) {
    begin_parse_count_session(root, false);
}

/// Count only parses on this thread.
///
/// Sequential tests that share fixture files with other lib tests must use this
/// so parallel threads cannot increment the session.
#[doc(hidden)]
pub fn begin_parse_count_this_thread(root: &Path) {
    begin_parse_count_session(root, true);
}

fn begin_parse_count_session(root: &Path, owner_only: bool) {
    parse_counts()
        .lock()
        .expect("parse-count mutex poisoned")
        .insert(
            root.to_path_buf(),
            ParseCountSession {
                owner: std::thread::current().id(),
                owner_only,
                counts: std::collections::HashMap::new(),
            },
        );
}

#[doc(hidden)]
pub fn finish_parse_count(root: &Path) -> std::collections::HashMap<PathBuf, usize> {
    parse_counts()
        .lock()
        .expect("parse-count mutex poisoned")
        .remove(root)
        .map(|session| session.counts)
        .unwrap_or_default()
}

pub(crate) fn record_parse_path(path: &Path) {
    let mut counts = parse_counts().lock().expect("parse-count mutex poisoned");
    let current_thread = std::thread::current().id();
    for (root, session) in counts.iter_mut() {
        // Synthetic parses conventionally use relative sentinel paths and may run on a
        // worker rather than the thread that opened the request observation. Only the owning
        // thread may attribute relative sentinels; observed worker parses must use paths rooted
        // in their request so parallel sessions cannot contaminate one another.
        let owns_relative_parse = path.is_relative() && session.owner == current_thread;
        let rooted = path.starts_with(root);
        if session.owner_only && session.owner != current_thread {
            continue;
        }
        if rooted || owns_relative_parse {
            *session.counts.entry(path.to_path_buf()).or_insert(0) += 1;
        }
    }
}
