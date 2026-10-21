// A stand-in isolates registration wiring from the external allocator library.
#[allow(non_upper_case_globals)]
mod mimalloc {
    pub type MiMalloc = std::alloc::System;
    pub const MiMalloc: MiMalloc = std::alloc::System;
}

// Real library unit-test allocation counters need one unique global allocator.
#[cfg(test)]
#[global_allocator]
static TEST_ALLOCATOR: std::alloc::System = std::alloc::System;

fn main() {}
