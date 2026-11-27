# `no-mistakes/playwright-defaults`

## Why

A component that passes a test-id prop into JSX needs a literal default so
static analysis can identify the selector when the caller omits the prop.

## Disallowed

```tsx
export function SaveButton({ testId }: { testId?: string }) {
  return <button data-pw={testId}>Save</button>;
}
```

```tsx
declare const id: string;

export function RowButton() {
  return <button {...{ "data-pw": id }}>Save</button>;
}
```

A spread the rule cannot read, and a spread that sets a selector attribute to
a non-literal, are `unanalyzable`. A literal selector spread is quiet.

## Allowed

```tsx
export function SaveButton({
  testId = "save-button",
  id,
}: {
  testId?: string;
  id?: string;
}) {
  return (
    <>
      <button data-pw={testId}>Save</button>
      <button data-pw={id ?? "save"}>Save</button>
    </>
  );
}
```

## Options

- `selectorAttributes` lists attributes treated as test IDs. It defaults to
  `["data-testid", "data-pw"]`.

## Fix

Give the passed-through prop a literal default, or use a literal test-id value
at the JSX site.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-defaults -- selector is supplied by a generated host
// no-mistakes-disable-next-line playwright-defaults -- selector is supplied by a generated host
// no-mistakes-disable-line playwright-defaults -- selector is supplied by a generated host
// no-mistakes-disable-file playwright-defaults -- selector is supplied by a generated host
return <button data-pw={testId}>Save</button>;
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not.

## Related rules

- [`playwright-literals`](playwright-literals.md) controls which dynamic forms
  remain acceptable.
