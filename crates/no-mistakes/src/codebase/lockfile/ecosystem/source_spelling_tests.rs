#[test]
fn gemfile_base64_sha256_checksums_match_hex() {
    let base = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "checksum-base64-base.lock"),
    )
    .unwrap();
    let encoded = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "checksum-base64-hex.lock"),
    )
    .unwrap();
    assert_eq!(base, encoded);
    expect(
        &base,
        "sample-b64",
        "1.0.0",
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        ResolutionKind::Registry,
    );
    expect(
        &base,
        "sample-url",
        "1.0.0",
        "fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1e0",
        ResolutionKind::Registry,
    );
    for name in ["sample-bad", "sample-junk", "sample-pad"] {
        expect(&base, name, "1.0.0", "", ResolutionKind::Registry);
    }
    let head = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "checksum-base64-head.lock"),
    )
    .unwrap();
    assert_eq!(diff(&base, &head).changed, ["sample-url"]);
}

#[test]
fn uv_accepts_implicit_pep440_spellings() {
    let packages = packages_for_diff(PackageManager::Uv, &fixture("uv", "spellings.lock")).unwrap();
    expect(
        &packages,
        "sample-implicit",
        "1.2a",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-pre",
        "1.0pre1",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-post",
        "1.0-1",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-joined",
        "1.2a.post1",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-label",
        "1.0-post1",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-dot",
        "1.2a.",
        "",
        ResolutionKind::Registry,
    );
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "spelling-post.lock")).is_err());
}
