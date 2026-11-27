# `no-mistakes/nextjs-no-manual-script-tags`

## Why

`next/script` supplies Next.js loading and ordering behavior that a raw JSX
`<script>` tag bypasses. Projects that centralize structured data in a component
can also opt in to reporting raw JSON-LD tags.

## Disallowed

```tsx
// app/page.tsx
export function Page() {
  return <script src="https://example.test/widget.js" />;
}
```

## Allowed

```tsx
// app/page.tsx
import Script from "next/script";

export function Page() {
  return <Script src="https://example.test/widget.js" strategy="afterInteractive" />;
}
```

## Options

- `allowInlineScriptIds` lists literal `id` values that exempt a raw `<script>`
  only when that tag also has `dangerouslySetInnerHTML`.
- `allowInlineScriptIdPatterns` lists regular-expression strings matched
  against that same literal `id` on a `<script>` that has
  `dangerouslySetInnerHTML`.

Both inline-id options default to no exemptions.

- `structuredDataComponent` is a nonempty component name to recommend instead
  of raw JSON-LD scripts, for example `"StructuredData"`. The name is used in
  the diagnostic; the rule does not resolve its import or validate the component.
- `structuredDataMessage` is a nonempty custom diagnostic for raw JSON-LD
  scripts. It overrides the generated component message when both are set.

A raw `<script>` with `type="application/ld+json"` or
`type={"application/ld+json"}` is allowed by default. Setting either structured
data option opts in to reporting those tags. Inline-id exemptions do not
bypass this opt-in. Dynamic type expressions and template literals retain the
ordinary raw-script behavior.

Use ESLint flat-config `files` and `ignores` for scope, including excluding the
implementation of the configured wrapper. Existing Next.js detection still
applies: the file must be under `app` or `pages`, or import from `next/`.

```js
{
  files: ["app/**/*.{jsx,tsx}"],
  ignores: ["app/components/structured-data.tsx"],
  rules: {
    "no-mistakes/nextjs-no-manual-script-tags": ["error", {
      structuredDataComponent: "StructuredData",
      structuredDataMessage: "Render JSON-LD with the structured-data component.",
    }],
  },
}
```

With this opt-in, a raw JSON-LD tag is disallowed:

```tsx
export function Page() {
  return <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: "{}" }} />;
}
```

Use the project's component instead:

```tsx
import { StructuredData } from "./components/structured-data";

export function Page() {
  return <StructuredData />;
}
```

## Fix

Replace executable scripts with `next/script`, or allow a
`dangerouslySetInnerHTML` script by a stable, reviewable `id` exemption.
When structured-data enforcement is enabled, replace raw JSON-LD tags with the
configured component or follow the custom diagnostic. Without that opt-in,
literal JSON-LD tags remain allowed. This rule does not autofix either case.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/nextjs-no-manual-script-tags -- required browser parser bootstrap
const bootstrap = <script id="parser-bootstrap">window.ready = true;</script>;
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not.

## Related rules

- [`nextjs-metadata-exports-location`](nextjs-metadata-exports-location.md)
  keeps Next.js conventions in route files.
