# `csp-allowed-origins`

## Why and when

Adding a third-party `<script>`, `<img>`, stylesheet `<link>`, `<iframe>`, media element, or `fetch`/`WebSocket`/`EventSource` origin in frontend code without updating the application's Content-Security-Policy (CSP) causes browsers to silently block the resource in production even when local development runs without a strict CSP header.

Enable `csp-allowed-origins` when your repository declares static CSP directives in one or more checked-in files (such as `src/csp.ts`, `next.config.ts`, `vercel.json`, or `public/_headers`) and you want `no-mistakes check` to fail whenever frontend code references an unlisted third-party network origin.

The rule is **opt-in** (disabled by default) and requires explicit `options.cspFiles`.

```yaml
version: 2
rules:
  - rule: csp-allowed-origins
    scope: repository
    include:
      - src/**/*.{ts,tsx}
    options:
      cspFiles:
        - src/csp.ts
      selfOrigins:
        - https://app.example.com
```

## What it catches

For each indexable JS/TS/JSX/TSX file matched by the rule's `include`/`exclude` filters (excluding `options.cspFiles` themselves), `csp-allowed-origins` inspects static network URLs (`http://`, `https://`, `ws://`, `wss://`) in:

- `<script src="...">` checked against `script-src`
- `<img src="...">` checked against `img-src`
- `<link href="...">` checked against `style-src` (`rel="stylesheet"`), `img-src` (`rel` containing `icon`), `script-src` (`rel="modulepreload"`), the matching `as` directive for `rel="preload"`, or `default-src` (skipping non-fetch hints such as `preconnect`, `dns-prefetch`, `canonical`, `alternate`)
- `<iframe src="...">` checked against `frame-src` (falling back to `child-src`, then `default-src`)
- `<audio src="...">`, `<video src="...">`, `<source src="...">` checked against `media-src`
- `fetch("...")`, `new WebSocket("...")`, and `new EventSource("...")` (including `window.`/`globalThis.`/`self.` qualifiers) checked against `connect-src`

Each origin is matched against the directive's parsed CSP source list (falling back to `default-src` per W3C CSP3), supporting exact origins, wildcard subdomains (`https://*.example.org`), scheme sources (`https:`), `*`, and `'self'` (matched against `options.selfOrigins`). Relative paths, `data:`, `blob:`, and dynamic expressions abstain without false positives.

## Options

- `cspFiles` (`string[]`, **required**): Non-empty list of repository-relative paths declaring static CSP directives. Supports JS/TS/JSX/TSX modules (read from static string/template literals in the prepared AST) as well as non-JS/TS files such as JSON, HTML, YAML, and `_headers`. Missing, unreadable, or directive-free files are reported as incomplete check inputs.
- `selfOrigins` (`string[]`, optional, default: `[]`): First-party origins (for example `https://app.example.com`) that satisfy `'self'` in the parsed CSP.

## Valid example

```ts
// src/csp.ts
export const contentSecurityPolicy = [
  "default-src 'self'",
  "img-src 'self' https://cdn.example.com https://*.assets.example.org",
  "connect-src 'self' https://api.example.com",
].join("; ");
```

```tsx
// src/page.tsx
export function Page() {
  void fetch("https://api.example.com/v1/items");
  return <img src="https://cdn.example.com/logo.png" alt="" />;
}
```

## Counterexample

```tsx
// src/page.tsx
export function Page() {
  // Reported: https://unlisted-cdn.example.net is not allowed by img-src
  return <img src="https://unlisted-cdn.example.net/hero.png" alt="" />;
}
```

## Fix

Add the third-party origin to the matching CSP directive in the file(s) listed in `options.cspFiles`, or remove the third-party origin and serve the asset from `'self'`:

```ts
// src/csp.ts
export const contentSecurityPolicy = [
  "default-src 'self'",
  "img-src 'self' https://cdn.example.com https://unlisted-cdn.example.net",
].join("; ");
```

## Suppression

Use `// no-mistakes-disable-file csp-allowed-origins` at the top of a file, `// no-mistakes-disable-line csp-allowed-origins` on the reported line, or `// no-mistakes-disable-next-line csp-allowed-origins` directly above it:

```ts
// no-mistakes-disable-next-line csp-allowed-origins
void fetch("https://preview-api.example.net/health");
```

## Node API

The same configured rule runs through async `check()` and `analyzeProject()` check reports. `CspAllowedOriginsOptions` is exported from `no-mistakes` for rule configuration (`cspFiles` and `selfOrigins`).

## Related rules

- [`nextjs-redirect-destinations`](nextjs-redirect-destinations.md) — validate Next.js redirect destinations against configured routes and allowed hosts.
- [`config-path-references`](config-path-references.md) — validate static path references inside structured config files.
