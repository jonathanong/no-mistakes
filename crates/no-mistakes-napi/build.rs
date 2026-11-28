fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
    println!("cargo::rustc-check-cfg=cfg(codspeed)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_all)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_0)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_1)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_2)");
    // Always compile every unit-test module here and stay independent of
    // `NM_TEST_SHARD` so shard 0 runs the complete no-mistakes-napi test suite.
    println!("cargo::rustc-cfg=nm_shard_all");
    napi_build::setup();
}
