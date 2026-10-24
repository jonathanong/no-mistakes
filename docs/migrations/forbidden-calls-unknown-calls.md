# `forbidden-calls` unknown call default

`forbidden-calls` now treats omitted `options.unknownCalls` as `finding`.
This changes check results: applications that previously omitted the option may
gain `unknown call` findings for selected unresolved, shadowed, computed, or
dynamic calls. A configuration with roots and no targets is now valid because
the default unknown-call policy can produce findings.

Review each new finding. Rewrite a dynamic call as a statically resolved call
when that is the intended boundary, or add a `no-mistakes-disable-next-line
forbidden-calls` directive with a reason for one intentional use. To preserve
the previous unmatched-call behavior for an application, set the opt-out
explicitly:

```yaml
rules:
  - rule: forbidden-calls
    scope: repository
    options:
      roots: [{ file: src/entry.ts }]
      targets: [{ global: setTimeout }]
      unknownCalls: ignore
```

`exact` and `terminal` selector matches still report under `ignore`; the
option only omits unmatched unknown calls. An application using `ignore` still
requires at least one target. This option is separate from the shared
`options.unanalyzable` policy.

The saved configurations in
`test-cases/rules/forbidden-calls/consumer-matrix/fixture/configs/`
show the default, explicit `finding`, and explicit `ignore` results on the same
dynamic call. Downstream consumers should validate the changed default with a
local `no-mistakes` binary before adopting the next release.
