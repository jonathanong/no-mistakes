# `playwright-unique-test-ids`

Requires unique configured test ID values in Playwright selector analysis.

```yaml
rules:
  - rule: playwright-unique-test-ids
    scope: repository
```

Counterexample: two components in the same analyzed surface use
`data-testid="save"`.

Fix: rename one selector or scope the components so coverage is unambiguous.

## Multiple frontend apps

The "same analyzed surface" is per rule application, not repository-wide: two
apps in a monorepo using the same `data-pw` value are **not** flagged when
each `playwright-unique-test-ids` application is bound to its own app (see
[Multiple frontend apps](../configuration/tests.md#multiple-frontend-apps)),
since each scan only ever sees its own app's occurrences. To intentionally
check uniqueness across apps, configure one repository-scoped rule
application with `tests.playwright.selectorRoots` set to their shared parent
directory.

## Why and when

Use this rule when `data-testid`, `data-pw`, or another configured test hook is
the stable contract between components and Playwright tests.

## What it catches/requires

Each configured test-ID value must identify at most one component in the rule's
analyzed surface. App bindings define the surface; duplicate values in two
independent apps are allowed when each rule application is app-scoped.

## Options and defaults

There are no uniqueness-specific options. The configured selector attributes
and roots under `tests.playwright` determine which values are considered.
Shared `effort` and `unanalyzable` options apply as described below.

## Effort

`effort: max` is the default. It recovers identifier, helper, ternary, object
map, and imported bindings into exact test IDs before judging uniqueness.

At effective `effort: low`, uniqueness accepts only:

- a string literal attribute, including JSX `data-testid={"save"}`
- a prop whose default is a string literal, matching `playwright-literals`

```tsx
<button data-testid="save-profile" />
<button data-testid={"cancel-profile"} />
function SaveButton({ testId = "save-button" }) {
  return <button data-testid={testId} />;
}
```

Every other shape is unanalyzable: an identifier without a literal default, a
ternary or logical leaf, a template, a helper call, or a cross-file binding.
The diagnostic names the rewrite:

```text
web/app/page.tsx:12: selector [data-testid="{id}"] is unanalyzable at effort low; use a literal test id such as data-pw="save-button", or a prop with a literal default
```

The shared `unanalyzable` policy controls that diagnostic: `report` fails,
`advise` emits a non-failing advisory, and `ignore` skips it. Rejected shapes
are never uniqueness evidence under any of those policies.

These syntax proofs do not require a companion rule. The rule checks each
selector once, so if any enabled application of this rule is not `low`, the
invocation stays at `max` and does not rewrite the other application's
findings. Identifier recovery is skipped only when every Playwright selector
consumer in the invocation is effectively `low`; a max `playwright-coverage`
application still recovers values, and this rule then treats recovered exact
IDs as unanalyzable rather than uniqueness hits.

Suppress a low-effort shape on its own line:

```tsx
// no-mistakes-disable-next-line playwright-unique-test-ids
<button data-testid={id} />
```

## Valid example

```tsx
// next.config.ts
export default {};

// app/page.tsx
export default function Page() {
  return (
    <>
      <button data-pw="save-profile">Save</button>
      <button data-pw="cancel-profile">Cancel</button>
    </>
  );
}
```

## Counterexample

```tsx
// next.config.ts
export default {};

// app/page.tsx
export default function Page() {
  return (
    <>
      <SaveButton data-pw="save" />
      <DeleteButton data-pw="save" />
    </>
  );
}
```

## Fix

Give each independently addressable control a distinct value, or bind the
rule to the correct frontend app/selector roots.

## Suppression

Use `no-mistakes-disable-line playwright-unique-test-ids`,
`no-mistakes-disable-next-line`, or `no-mistakes-disable-file` for an
intentional compatibility alias. Prefer app scoping when the duplicate is
cross-application rather than a true collision.

## Limitations

Playwright file discovery skips a directory segment named `.git`,
`node_modules`, `target`, `dist`, `build`, `coverage`, or `test-results`,
even when the file is tracked. Duplicate test IDs that live only under
`src/components/build/` are not reported.

```tsx
// src/components/build/SaveButton.tsx and src/components/build/DeleteButton.tsx
<button data-pw="save" />
<button data-pw="save" />
```

## Related rules

[`playwright-prefer-test-id-locators`](playwright-prefer-test-id-locators.md)
encourages using these hooks; [`playwright-coverage`](playwright-coverage.md)
checks that tested hooks are actually exercised.
