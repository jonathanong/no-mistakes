#![expect(
    clippy::disallowed_methods,
    reason = "Invocation deadlines require a control clock independent of optional analysis instrumentation."
)]
use std::time::Instant;

pub(super) fn now() -> Instant {
    Instant::now()
}
