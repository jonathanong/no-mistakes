use super::super::Deadline;
use super::pauses;
mod encoded;
mod offset;
pub(in crate::invocation::deadline) use encoded::Encoded;
use offset::{encode_instant, encode_timeout};
use std::collections::HashMap;
use std::sync::atomic::{fence, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, PoisonError};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

/// Lock-free deadline image. Every shared word is an `AtomicU64`.
///
/// Writers store an odd generation, then these words, then the next even
/// generation. `Release` fences keep that store order. Readers `Acquire`-load
/// an even generation, copy the words, and `Acquire`-load the generation again.
/// A matching pair means the copy is one published image: the even store's
/// preceding `Release` fence synchronizes with that first `Acquire` load, and
/// the second load rejects a writer that entered during the copy. Relaxed
/// payload accesses are atomic, so a rejected copy is a torn image, not a race.
struct Words {
    generation: AtomicU64,
    present: AtomicU64,
    /// Signed seconds from the process epoch. Nanoseconds do not fit in `i64`.
    expires_secs: AtomicU64,
    timeout_secs: AtomicU64,
    /// Low 32 bits are the expiry subsec; high 32 bits are the timeout subsec.
    subsec_nanos: AtomicU64,
    owner_token: AtomicU64,
    committed: AtomicU64,
}

static WORDS: Words = Words {
    generation: AtomicU64::new(0),
    present: AtomicU64::new(0),
    expires_secs: AtomicU64::new(0),
    timeout_secs: AtomicU64::new(0),
    subsec_nanos: AtomicU64::new(0),
    owner_token: AtomicU64::new(0),
    committed: AtomicU64::new(0),
};

static NEXT_OWNER_TOKEN: AtomicU64 = AtomicU64::new(1);
static OWNER_TOKENS: LazyLock<Mutex<HashMap<ThreadId, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(in crate::invocation::deadline) struct Observed {
    pub(in crate::invocation::deadline) expires_at: Instant,
    pub(in crate::invocation::deadline) timeout: Duration,
    pub(in crate::invocation::deadline) committed: bool,
    owner_token: u64,
}

impl Observed {
    /// `None` owners publish token 0 and do not touch the token map.
    pub(in crate::invocation::deadline) fn applies_here(&self) -> bool {
        if self.committed {
            return false;
        }
        self.owner_token == 0 || self.owner_token == current_owner_token()
    }
}

pub(super) fn publish(deadline: Option<Deadline>) {
    let generation = WORDS.generation.load(Ordering::Relaxed).wrapping_add(1);
    WORDS.generation.store(generation, Ordering::Relaxed);
    // The odd store stays before the payload stores.
    fence(Ordering::Release);
    pauses::after_odd_publish();
    store_words(deadline);
    // The payload stores stay before the even generation store.
    fence(Ordering::Release);
    WORDS
        .generation
        .store(generation.wrapping_add(1), Ordering::Relaxed);
}

pub(super) fn load() -> Option<Observed> {
    load_encoded().map(Encoded::decode)
}

pub(in crate::invocation::deadline) fn load_encoded() -> Option<Encoded> {
    loop {
        let first = WORDS.generation.load(Ordering::Acquire);
        if first & 1 != 0 {
            pauses::on_odd_generation();
            std::hint::spin_loop();
            continue;
        }
        // The first Acquire load already imports this even publication and
        // orders its payload reads; an additional Acquire fence is redundant.
        let present = WORDS.present.load(Ordering::Relaxed);
        let expires_secs = WORDS.expires_secs.load(Ordering::Relaxed);
        pauses::between_words();
        let timeout_secs = WORDS.timeout_secs.load(Ordering::Relaxed);
        let (expires_nanos, timeout_nanos) =
            unpack_subsec(WORDS.subsec_nanos.load(Ordering::Relaxed));
        let owner_token = WORDS.owner_token.load(Ordering::Relaxed);
        let committed = WORDS.committed.load(Ordering::Relaxed);
        // Payload loads stay before the generation load that accepts them.
        fence(Ordering::Acquire);
        pauses::before_second_load();
        let second = WORDS.generation.load(Ordering::Acquire);
        if first != second {
            std::hint::spin_loop();
            continue;
        }
        if present == 0 {
            return None;
        }
        return Some(Encoded {
            expires_secs: expires_secs as i64,
            expires_nanos,
            timeout_secs,
            timeout_nanos,
            committed: committed != 0,
            owner_token,
        });
    }
}

fn store_words(deadline: Option<Deadline>) {
    let Some(deadline) = deadline else {
        WORDS.present.store(0, Ordering::Relaxed);
        WORDS.expires_secs.store(0, Ordering::Relaxed);
        WORDS.timeout_secs.store(0, Ordering::Relaxed);
        WORDS.subsec_nanos.store(0, Ordering::Relaxed);
        WORDS.owner_token.store(0, Ordering::Relaxed);
        WORDS.committed.store(0, Ordering::Relaxed);
        return;
    };
    let (expires_secs, expires_nanos) = encode_instant(deadline.expires_at);
    let (timeout_secs, timeout_nanos) = encode_timeout(deadline.timeout);
    WORDS.present.store(1, Ordering::Relaxed);
    WORDS
        .expires_secs
        .store(expires_secs as u64, Ordering::Relaxed);
    WORDS.timeout_secs.store(timeout_secs, Ordering::Relaxed);
    WORDS
        .subsec_nanos
        .store(pack_subsec(expires_nanos, timeout_nanos), Ordering::Relaxed);
    WORDS
        .owner_token
        .store(owner_token(deadline.owner), Ordering::Relaxed);
    WORDS
        .committed
        .store(u64::from(deadline.committed), Ordering::Relaxed);
}

fn pack_subsec(expires_nanos: u32, timeout_nanos: u32) -> u64 {
    u64::from(expires_nanos) | (u64::from(timeout_nanos) << 32)
}

fn unpack_subsec(packed: u64) -> (u32, u32) {
    (packed as u32, (packed >> 32) as u32)
}

/// Tokens stand in for `ThreadId`, which has no stable integer conversion.
/// Registration happens on the write path. Unowned deadlines never lock this map.
fn owner_token(owner: Option<ThreadId>) -> u64 {
    let Some(owner) = owner else {
        return 0;
    };
    let mut tokens = OWNER_TOKENS.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(token) = tokens.get(&owner) {
        return *token;
    }
    let token = NEXT_OWNER_TOKEN.fetch_add(1, Ordering::Relaxed);
    tokens.insert(owner, token);
    token
}

fn current_owner_token() -> u64 {
    let id = std::thread::current().id();
    OWNER_TOKENS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&id)
        .copied()
        .unwrap_or(0)
}
