fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
    println!("cargo::rustc-check-cfg=cfg(codspeed)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_all)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_0)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_1)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_2)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_postgres_source)");
    println!("cargo:rerun-if-env-changed=NM_TEST_SHARD");
    println!("cargo::rustc-cfg={}", test_shard_cfg());
}

/// Unset `NM_TEST_SHARD` compiles every unit-test module. `0`, `1`, or `2`
/// compiles one standalone archive; `postgres-source` compiles only the
/// PostgreSQL source-identity unit tests.
fn test_shard_cfg() -> String {
    match std::env::var("NM_TEST_SHARD") {
        Err(_) => "nm_shard_all".to_string(),
        Ok(shard) if matches!(shard.as_str(), "0" | "1" | "2") => format!("nm_shard_{shard}"),
        Ok(shard) if shard == "postgres-source" => "nm_shard_postgres_source".to_string(),
        Ok(_) => panic!("NM_TEST_SHARD must be 0, 1, 2, or postgres-source"),
    }
}
