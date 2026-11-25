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
effective `href` is a JSX string or string literal expression. A property
spread after a literal `href` makes it uncertain. A spread before it preserves
the literal. It follows direct children through JSX fragments and the native
elements `a`, `button`, `div`, `em`, `span`, and `strong`.

## Options

This rule is off by default. Use the common `projects` or `scope` and
`include` / `exclude` filters. There are no rule-specific options. The check
runs at `max` effort. Selected JSX parse failures report an incomplete check
with the file and parse error.

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
elements. Use the surrounding layout for row styling.

## Suppression

Place `no-mistakes-disable-next-line react-no-nested-buttons-and-links`
immediately above the inner element or use `no-mistakes-disable-line` on that
line. Use `no-mistakes-disable-file react-no-nested-buttons-and-links` for a
whole-file exception. Use JavaScript line comments for line and next-line
directives on the same line as, or immediately above, the reported opening tag.

## Limits

This check covers the authored HTML ancestry described above. Component
rendering, child expressions, SVG and MathML subtrees, dynamic or spread-only
link destinations, other interactive native tags, and ARIA or handler-based
controls are outside that scope.

## Related rules

- [`playwright-require-interactive-test-id`](../eslint-rules/playwright-require-interactive-test-id.md)
  requires test hooks on interactive JSX elements.
