fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
    println!("cargo:rerun-if-env-changed=NO_MISTAKES_BUILD_NAPI");
    if std::env::var_os("NO_MISTAKES_BUILD_NAPI").is_some() {
        napi_build::setup();
    }
}
