use super::super::Deadline;
use super::pauses;
mod encoded;
pub(in crate::invocation::deadline) use encoded::Encoded;
use std::collections::HashMap;
use std::sync::atomic::{fence, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};
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
    expires_nanos: AtomicU64,
    timeout_nanos: AtomicU64,
    owner_token: AtomicU64,
    committed: AtomicU64,
}

static WORDS: Words = Words {
    generation: AtomicU64::new(0),
    present: AtomicU64::new(0),
    expires_nanos: AtomicU64::new(0),
    timeout_nanos: AtomicU64::new(0),
    owner_token: AtomicU64::new(0),
    committed: AtomicU64::new(0),
};

static EPOCH: OnceLock<Instant> = OnceLock::new();
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
        // Payload loads stay after the even generation that published them.
        fence(Ordering::Acquire);
        let present = WORDS.present.load(Ordering::Relaxed);
        let expires_nanos = WORDS.expires_nanos.load(Ordering::Relaxed);
        pauses::between_words();
        let timeout_nanos = WORDS.timeout_nanos.load(Ordering::Relaxed);
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
            expires_nanos,
            timeout_nanos,
            committed: committed != 0,
            owner_token,
        });
    }
}

fn store_words(deadline: Option<Deadline>) {
    let Some(deadline) = deadline else {
        WORDS.present.store(0, Ordering::Relaxed);
        WORDS.expires_nanos.store(0, Ordering::Relaxed);
        WORDS.timeout_nanos.store(0, Ordering::Relaxed);
        WORDS.owner_token.store(0, Ordering::Relaxed);
        WORDS.committed.store(0, Ordering::Relaxed);
        return;
    };
    WORDS.present.store(1, Ordering::Relaxed);
    WORDS
        .expires_nanos
        .store(encode_instant(deadline.expires_at), Ordering::Relaxed);
    WORDS.timeout_nanos.store(
        u64::try_from(deadline.timeout.as_nanos()).unwrap_or(u64::MAX),
        Ordering::Relaxed,
    );
    WORDS
        .owner_token
        .store(owner_token(deadline.owner), Ordering::Relaxed);
    WORDS
        .committed
        .store(u64::from(deadline.committed), Ordering::Relaxed);
}

fn epoch() -> Instant {
    *EPOCH.get_or_init(Instant::now)
}

fn encode_instant(instant: Instant) -> u64 {
    let epoch = epoch();
    let signed = if instant >= epoch {
        i64::try_from(instant.duration_since(epoch).as_nanos()).unwrap_or(i64::MAX)
    } else {
        let before = epoch.duration_since(instant).as_nanos();
        -i64::try_from(before).unwrap_or(i64::MAX)
    };
    signed as u64
}

fn decode_instant(bits: u64) -> Instant {
    let signed = bits as i64;
    let epoch = epoch();
    if signed >= 0 {
        epoch
            .checked_add(Duration::from_nanos(signed as u64))
            .unwrap_or(epoch)
    } else {
        epoch
            .checked_sub(Duration::from_nanos(signed.unsigned_abs()))
            .unwrap_or(epoch)
    }
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
