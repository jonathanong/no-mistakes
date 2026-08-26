# `no-mistakes/nextjs-no-manual-script-tags`

## Why

`next/script` supplies Next.js loading and ordering behavior that a raw JSX
`<script>` tag bypasses.

## Disallowed

```tsx
export function Page() {
  return <script src="https://example.test/widget.js" />;
}
```

## Allowed

```tsx
import Script from "next/script";

export function Page() {
  return <Script src="https://example.test/widget.js" strategy="afterInteractive" />;
}
```

## Options

- `allowInlineScriptIds` lists literal `id` values allowed on inline raw
  scripts.
- `allowInlineScriptIdPatterns` lists regular-expression strings that allow an
  inline raw script id.

Both options default to no exemptions.

## Fix

Replace the tag with `next/script`. Allow a raw inline script only by a stable,
reviewable `id` exemption.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/nextjs-no-manual-script-tags -- required browser parser bootstrap
const bootstrap = <script id="parser-bootstrap">window.ready = true;</script>;
```

## Related rules

- [`nextjs-metadata-exports-location`](nextjs-metadata-exports-location.md)
  keeps Next.js conventions in route files.
