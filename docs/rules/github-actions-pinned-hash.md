# `github-actions-pinned-hash`

Requires every `uses:` step in GitHub Actions workflow files to reference a
40-char commit SHA with a version comment. Tag- or branch-pinned actions are
mutable — an upstream owner can move the ref at any time, silently swapping
the action's code.

```yaml
rules:
  - rule: github-actions-pinned-hash
    scope: repository
```

**Pass:**

```yaml
- uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
- uses: dtolnay/rust-toolchain@6190aa5fb88a88ee71c12769924bbe63a9ab152e # 1.96.0
```

**Counterexample:**

```yaml
- uses: actions/checkout@v6.0.2         # tag — fails
- uses: dtolnay/rust-toolchain@stable   # branch — fails
- uses: actions/checkout@de0fac2e45...  # SHA but no version comment — fails
- uses: actions/checkout@de0fac2e45...  # main  # comment not a version — fails
```

**Fix:** Pin to the commit SHA for the desired tag and record the version in a
trailing comment:

```bash
gh api repos/actions/checkout/commits/v6.0.2 --jq '.sha'
```

Then update the workflow line:

```yaml
- uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
```

The comment must start with a version number (`# v1.2.3`, `# v2`, or
`# 1.96.0`). Branch-name and tool-name comments (`# main`, `# stable`) are
rejected.

**Exemptions:** Local action references (`./`, `../`) and Docker image
references (`docker://`) are never checked. Use `excludePaths` to opt out
specific files:

```yaml
rules:
  - rule: github-actions-pinned-hash
    scope: repository
    options:
      excludePaths:
        - .github/workflows/release.yml
```

Use `no-mistakes-disable-next-line github-actions-pinned-hash` to suppress a
single line.

**Scope:** Checks `.github/workflows/*.{yml,yaml}` and
`.github/actions/**/action.{yml,yaml}` composite action files.
