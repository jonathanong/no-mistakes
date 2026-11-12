use std::time::Instant as Clock;

#[expect(clippy::disallowed_methods, reason = "Verify type-resolved qualified clock rejection.")]
pub fn qualified() -> Clock {
    std::time::Instant::now()
}

#[expect(clippy::disallowed_methods, reason = "Verify imported clock aliases cannot bypass policy.")]
pub fn aliased() -> Clock {
    Clock::now()
}

#[expect(clippy::disallowed_methods, reason = "Verify the timing owner's documented module exemption.")]
pub mod owner {
    pub fn clock() -> std::time::Instant {
        std::time::Instant::now()
    }
}
