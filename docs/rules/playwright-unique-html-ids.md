# `playwright-unique-html-ids`

Requires unique HTML `id` values in Playwright selector analysis.

```yaml
rules:
  - rule: playwright-unique-html-ids
    scope: repository
```

Counterexample: two rendered elements use `id="submit"`.

Fix: make IDs unique, suppress the intentional finding, or disable the rule for
the relevant target.

Use `no-mistakes-disable-file`, `no-mistakes-disable-line`, or
`no-mistakes-disable-next-line` for intentional exceptions; `htmlIds: false` is
not a suppression directive.

This rule scans HTML IDs independently of
[`tests.playwright.selectors.htmlIds`](../configuration/tests.md). Setting
`htmlIds: false` keeps IDs out of `playwright-coverage`, but does not suppress
duplicate-ID findings from `playwright-unique-html-ids`.
