# Checkout credentials: unknown inputs now report by default

[`github-actions-checkout-credentials`](../rules/github-actions-checkout-credentials.md)
used to silently skip checkout credentials it could not analyze. With the rule
already listed, the shared `options.unanalyzable` policy now defaults to
`report`: unknown inputs become failing findings in CLI and programmatic checks.
The rule remains opt-in.

When credentials might persist, review these previously silent cases:

- Missing `permissions`: repository or organization defaults are unknown.
  `pull_request_target` alone does not prove the actual token grant either.
- An explicit write grant in a `workflow_call` workflow: the caller can
  restrict the token, so this file does not establish its effective grant.
- A custom `with.token`, such as a PAT or `${{ secrets.GITHUB_TOKEN }}`:
  job permissions describe the default `GITHUB_TOKEN`, not a PAT's permissions,
  and a custom token expression does not establish its identity or permissions.
- Expression-valued `persist-credentials`: the value is determined at runtime
  and cannot be established statically.
- Invalid YAML: the parser cannot establish the checkout inputs and permissions.

Other unsupported permission and persistence shapes follow the same policy.
Literal `with.persist-credentials: false` remains safe, including with a custom
token. A null `with` is treated like an omitted `with`; other non-mapping `with`
values are unknown.

The [reviewed unchanged old `abstain` fixture/configuration snapshot][reviewed-abstain]
exited 0 with no findings on the base and exited 1 with 36 findings on the head.
This is evidence for that snapshot, not an expected finding count for every
repository. The current `abstain` matcher fixture explicitly opts into `ignore`,
so its passing result does not preserve or demonstrate the old default policy.

## Migrate an existing configuration

Review each new finding and prefer a static rewrite:

- Set `with.persist-credentials: false` when later steps do not need stored
  credentials.
- For the default token, use literal read-only permissions when writes are not
  needed. Unknown persistence still follows the policy even when those
  permissions prove the token cannot write; this conservative choice is
  intentional. Literal false removes that uncertainty.
- Keep custom tokens out of the git credential store with literal false.
- Repair invalid YAML before relying on the analysis.

To skip only unknown inputs, set `ignore` explicitly:

```yaml
rules:
  - rule: github-actions-checkout-credentials
    scope: repository
    options:
      unanalyzable: ignore
```

Use `unanalyzable: advise` instead to list unknown inputs as non-failing
advisories in aggregate CLI and Node check reports. The standalone Rust
findings-only API omits advisories; see the
[rule API contract](../rules/github-actions-checkout-credentials.md#suppression).
Advisory output does not reproduce the old output byte for byte. Both policies
continue to report definite persisted-write violations. Neither policy
guarantees complete report parity with an older version.

With `assumeWriteWhenUnspecified: true`, missing permissions are a definite
write assumption, including in a `workflow_call` workflow. These remain
failures under `advise` and `ignore`. An explicit write grant in a
`workflow_call` file instead remains unknown because its caller can restrict
it. Job permissions replace workflow permissions.

## Accept one checkout without changing the whole rule

Place a suppression on the checkout `uses` line, or immediately above it:

```yaml
# no-mistakes-disable-next-line github-actions-checkout-credentials
- uses: actions/checkout@v4
  with:
    token: ${{ secrets.RELEASE_PAT }}
```

`no-mistakes-disable-line github-actions-checkout-credentials` suppresses the
same line. `no-mistakes-disable-file github-actions-checkout-credentials` in
the leading comment block suppresses a whole file. These directives apply to
findings and advisories. For invalid YAML, the reported line is the parser-error
location rather than a checkout `uses` line.

[reviewed-abstain]: https://github.com/jonathanong/no-mistakes/pull/2697#issuecomment-6383196957
