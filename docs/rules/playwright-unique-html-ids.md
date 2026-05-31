# `playwright-unique-html-ids`

Requires unique HTML `id` values in Playwright selector analysis.

```yaml
rules:
  - rule: playwright-unique-html-ids
    scope: repository
```

Counterexample: two rendered elements use `id="submit"`.

Fix: make IDs unique or disable HTML ID selector tracking if the project does
not use IDs for tests.
