fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
    println!("cargo::rustc-check-cfg=cfg(codspeed)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_all)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_0)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_1)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_2)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_postgres_source)");
    println!("cargo:rerun-if-env-changed=NM_POSTGRES_SQL_TEST_SHARD");
    println!("cargo::rustc-cfg={}", test_shard_cfg());
}

/// Unset `NM_POSTGRES_SQL_TEST_SHARD` compiles every unit-test module and stays
/// unaffected when `NM_TEST_SHARD` cycles across `no-mistakes` shards `0..=2`,
/// so Cargo reuses this crate's compiled `rlib` without recompiling.
/// `postgres-source` compiles only the source-identity unit tests.
fn test_shard_cfg() -> &'static str {
    match std::env::var("NM_POSTGRES_SQL_TEST_SHARD").as_deref() {
        Err(_) => "nm_shard_all",
        Ok("postgres-source") => "nm_shard_postgres_source",
        Ok(_) => panic!("NM_POSTGRES_SQL_TEST_SHARD must be postgres-source"),
    }
}
