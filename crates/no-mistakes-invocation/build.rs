fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
    println!("cargo::rustc-check-cfg=cfg(codspeed)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_all)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_0)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_1)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_2)");
    println!("cargo::rustc-check-cfg=cfg(nm_shard_postgres_source)");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-cfg=nm_shard_all");
}
