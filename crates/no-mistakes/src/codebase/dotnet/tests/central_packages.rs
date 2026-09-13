use super::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[test]
fn central_package_imports_cover_unclosed_comments_msbuild_and_ancestors() {
    let nested = PathBuf::from("/repo/src/Directory.Packages.props");
    let parent = PathBuf::from("/repo/Directory.Packages.props");
    let sibling = PathBuf::from("/repo/src/Packages.props");
    let files = BTreeSet::from([nested.clone(), parent.clone(), sibling.clone()]);

    assert!(
        central_package_imports(&nested, "<!-- unterminated", &files).contains(&parent),
        "unclosed comment should fall back to ancestor files"
    );
    assert!(central_ancestor_files(Path::new("/"), &files).is_empty());

    let imported =
        central_package_imports(&nested, r#"<Import Project="Packages.props" />"#, &files);
    assert_eq!(imported, vec![sibling.clone()]);

    let above = central_package_imports(
        &nested,
        r#"<Import Project="$([MSBuild]::GetPathOfFileAbove('Directory.Packages.props', '$(MSBuildThisFileDirectory)..'))" />"#,
        &BTreeSet::from([nested.clone(), parent.clone()]),
    );
    assert_eq!(above, vec![parent.clone()]);

    assert!(central_package_imports(
        &nested,
        r#"<Import Project="$(MSBuildThisFileDirectory)Packages.props" />"#,
        &files,
    )
    .is_empty());

    let commented = central_package_imports(
        &nested,
        r#"<!-- <Import Project="Packages.props" /> --><Import Project="..\Directory.Packages.props" />"#,
        &files,
    );
    assert_eq!(commented, vec![parent.clone()]);

    let cdata = central_package_imports(
        &nested,
        r#"<![CDATA[<Import Project="Packages.props" />]]><Import Project="..\Directory.Packages.props" />"#,
        &files,
    );
    assert_eq!(cdata, vec![parent]);
}
