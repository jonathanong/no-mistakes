# `no-mistakes/playwright-require-interactive-test-id`

## Why

Interactive controls are high-value test targets. A stable test ID makes their
coverage and automation intent explicit.

## Disallowed

```tsx
<button onClick={save}>Save</button>
```

## Allowed

```tsx
<button data-pw="save-button" onClick={save}>Save</button>
```

A configured interactive component may forward its test ID onto one direct
element through a Radix-style slot:

```tsx
// interactiveComponents: ["Button"]
<Button asChild data-pw="download-original">
  <a href={url} download>Download</a>
</Button>
```

The parent must carry a literal selector attribute. A missing ID, dynamic
value, or spread-only ID is reported on the parent; the single child does not
receive a duplicate finding. Ordinary, non-slot elements keep their existing
selector-presence behavior. Use `playwright-literals` to validate their values.

## Options

- `selectorAttributes` lists acceptable test-id attributes; it defaults to
  `["data-testid", "data-pw"]`.
- `interactiveComponents` adds component names considered interactive. Entries
  may be exact names or `/regex/` patterns.
- `slotProps` lists slot-mode props on those configured components. It defaults
  to `["asChild"]`; an explicit list replaces that default, and `[]` disables
  slot recognition. Shorthand props and statically truthy literals enable slot
  mode; false, null, zero, empty strings, and dynamic expressions do not.

The slot example above uses the default slot prop and selector attributes:

```js
{ interactiveComponents: ["Button"] }
```

For a different slot prop and selector attribute:

```js
{
  "no-mistakes/playwright-require-interactive-test-id": ["error", {
    interactiveComponents: ["Button"],
    slotProps: ["renderChild"],
    selectorAttributes: ["data-qa"],
  }],
}
```

## Fix

Add a configured literal test ID to the interactive element, or configure a
project-specific interactive component name.

For a slot, add the literal ID to the parent. If a spread precedes the slot or
selector prop, write the explicit prop after that spread so its value is known.

## Limits

Slot inheritance applies only to components matched by `interactiveComponents`,
using the same import/alias matching as ordinary checks. Native elements and
components outside that list do not establish a slot contract. Only the immediate,
single direct JSX element is covered. Newline indentation and JSX comments are
ignored; text siblings (including same-line spaces), expression children,
fragments, and multiple elements leave the children checked independently.
Nested interactive descendants still need their own IDs.

Later duplicate props or spreads can override earlier props. An unknown spread
after the slot prop prevents proving slot mode. A literal selector must follow
any parent spread to establish that selector. A child spread may override an
inherited ID, so the child remains checked unless it carries its own selector.
Dynamic slot expressions are not evaluated or followed through variables.

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A nested component that reuses an imported local name is that local component, not the import. `const Alias = Save` is the import only when that `Save` binding is the import, so `function Wrapper(Save) { const Alias = Save; return <Alias />; }` is not `SaveButton`. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```tsx
// controls.tsx
export { Button as Action } from "./button";
```

```tsx
// form.tsx
import { Action } from "./controls";
// With interactiveComponents: ["Button"], Action cannot be traced to Button here.
export function Form() { return <Action />; }
```

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-require-interactive-test-id -- decorative control is deliberately not automation-facing
const decorative = <button onClick={animate}>Play</button>;
```

## Related rules

- [`playwright-require-exported-component-attribute`](playwright-require-exported-component-attribute.md)
  covers exported component roots and branches.
