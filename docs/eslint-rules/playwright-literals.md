# `no-mistakes/playwright-literals`

Requires literal Playwright selector values.

Why: literal selectors allow coverage, related-test, and uniqueness analysis to
map tests to UI targets.

Counterexample: `page.getByTestId(buttonId)` or `<button data-pw={id} />`.

Fix: use string literals or supported static templates for test IDs and related
selector arguments.

Example:

```tsx
page.getByTestId("save-button");
<button data-pw="save-button" />;
```

Counterexample:

```tsx
page.getByTestId(buttonId);
<button data-pw={props.testId} />;
```

Options:

```js
{
  selectorAttributes: ["data-testid", "data-pw"],
  allowDefaultedProps: true,
  allowStaticTemplates: false,
}
```

Caveat: literal default prop values are accepted by default so component APIs can
carry stable test IDs. Set `allowDefaultedProps: false` when the project wants
only directly written literals.
