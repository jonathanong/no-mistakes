fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
    println!("cargo::rustc-check-cfg=cfg(codspeed)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_all)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_0)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_1)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_2)");
    // Always compile every unit-test module here and stay independent of
    // `NM_TEST_SHARD` so Cargo reuses this crate's `rlib` across all three
    // `no-mistakes` standalone shards without recompiling.
    println!("cargo::rustc-cfg=nm_shard_all");
}
