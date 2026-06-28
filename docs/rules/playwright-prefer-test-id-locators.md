# `playwright-prefer-test-id-locators`

Flags Playwright text locators that could use an existing configured test ID.

```yaml
rules:
  - rule: playwright-prefer-test-id-locators
    tests:
      playwright: [web]
```

Counterexample: a spec clicks `page.getByRole("button", { name: "Save" })`
when the matched app element exposes `data-pw="save-button"`.

Fix: use `page.getByTestId("save-button")`, or suppress the line when the
copy-coupled locator is intentional.

This rule is graph-backed. It only reports when Playwright route or adjacent
selector analysis can connect the locator to an app element with a configured
test ID.
