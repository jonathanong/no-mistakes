# `react-no-nested-buttons-and-links`

Reports native buttons and statically linked anchors nested inside another
button or statically linked anchor in JSX or TSX.

```yaml
version: 2
rules:
  - rule: react-no-nested-buttons-and-links
    projects: [web]
```

## Why and when

Enable this check in React HTML source. The HTML content model excludes
interactive descendants from buttons and anchors. Nested controls can make
keyboard activation or focus ambiguous, and invalid tag nesting can contribute
to hydration mismatches. [HTML buttons](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element),
[HTML anchors](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element),
[React hydration error](https://react.dev/errors/418).

## What it catches

The rule checks native `<button>` elements and native `<a>` elements whose
effective `href` is a JSX string or string literal expression. A spread before a
literal `href` preserves the literal, and `href={null}` clears a preceding
`href`. It follows direct children through JSX fragments and the native elements
`a`, `button`, `div`, `em`, `span`, and `strong`.

Shapes the rule cannot statically prove safe are reported through the
`unanalyzable` policy (`report` by default, or `advise` / `ignore`) rather than
passed silently:

- An `<a>` with a non-literal `href`, value-less `href`, or trailing attribute
  spread when it sits inside a control or encloses a control or unanalyzable
  child.
- A self-closing `<button>` or `<a>` with a spread attribute that could supply
  `children`.
- A non-empty JSX child expression or spread child inside a tracked button or
  link.
- Any component, member tag, or native wrapper element outside `a`, `button`,
  `div`, `em`, `span`, and `strong` (other than `svg` and `math`) inside a
  tracked button or link.

## Options

This rule is off by default. Use the common `projects` or `scope` and
`include` / `exclude` filters, plus the shared `options.unanalyzable`
(`report` | `advise` | `ignore`, inheriting repository `unanalyzable` and
defaulting to `report`). The check runs at `max` effort. Selected JSX parse
failures report an incomplete check with the file and parse error; a file that
could not be parsed does not produce `unused-disable-directive` advisories.

The existing async `check()` Node API and `analyzeProject()` check mode use
the same fact collection and rule report as the CLI.

## Valid example

```tsx
export function ItemActions() {
  return <div><a href="/items">Items</a><button type="button">Save</button></div>;
}
```

## Counterexample

```tsx
export function ItemActions() {
  return <a href="/items"><span><button type="button">Save</button></span></a>;
}
```

## Fix

Move the inner control outside the outer control and render them as sibling
elements. Use the surrounding layout for row styling. When an unanalyzable shape
is reported, rewrite it to a static literal `href` and supported wrappers,
suppress the line with a reason, or configure `options.unanalyzable`.

## Suppression

Use `no-mistakes-disable-line` on the line of the inner opening tag or
`no-mistakes-disable-next-line` one line above it when JavaScript permits a
`//` comment in that position.

```tsx
export const nextLine = (
  // no-mistakes-disable-next-line react-no-nested-buttons-and-links -- legacy UI
  <a href="/items"><button type="button">Save</button></a>
);
```

A `//` line inside JSX children renders as page text. When the opening tag
has no suitable JavaScript comment position, restructure the markup or use
`no-mistakes-disable-file react-no-nested-buttons-and-links`.

## Limits

This check proves native `<button>` and `<a>` ancestry within one file.
SVG and MathML subtrees, other interactive native tags, and ARIA or
handler-based controls are outside that scope. Component wrappers, child
expressions, spreads, and uncertain `href` values inside or around controls are
surfaced as unanalyzable instead of passed silently.

## Related rules

- [`playwright-require-interactive-test-id`](../eslint-rules/playwright-require-interactive-test-id.md)
  requires test hooks on interactive JSX elements.
