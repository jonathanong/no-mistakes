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
    let jsx_only = [PathBuf::from("src/helper.jsx")].into_iter().collect();
    assert_eq!(
        resolve_supplied_relative(from, "./helper.js", &jsx_only),
        Some(PathBuf::from("src/helper.jsx"))
    );
    let literal_and_jsx = [
        PathBuf::from("src/helper.js"),
        PathBuf::from("src/helper.jsx"),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        resolve_supplied_relative(from, "./helper.js", &literal_and_jsx),
        Some(PathBuf::from("src/helper.js"))
    );
}
