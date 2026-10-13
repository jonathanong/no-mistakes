use super::resolve_supplied_relative;
use std::path::{Path, PathBuf};

#[test]
fn supplied_node_next_imports_prefer_source_and_support_commonjs() {
    let files = [
        "src/helper.ts",
        "src/helper.js",
        "src/common.cts",
        "src/index.cjs",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect();
    let from = Path::new("src/consumer.mts");
    assert_eq!(
        resolve_supplied_relative(from, "./helper.js", &files),
        Some(PathBuf::from("src/helper.ts"))
    );
    assert_eq!(
        resolve_supplied_relative(from, "./common.cjs", &files),
        Some(PathBuf::from("src/common.cts"))
    );
    assert_eq!(
        resolve_supplied_relative(from, "./common", &files),
        Some(PathBuf::from("src/common.cts"))
    );
}
