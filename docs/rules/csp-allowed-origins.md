# `csp-allowed-origins`

## Why and when

Adding a third-party `<script>`, `<img>`, stylesheet `<link>`, `<iframe>`, media element, or `fetch`/`WebSocket`/`EventSource` origin in frontend code without updating the application's Content-Security-Policy (CSP) causes browsers to silently block the resource in production even when local development runs without a strict CSP header.

Enable `csp-allowed-origins` when your repository declares static CSP directives in one or more checked-in files (such as `src/csp.ts`, `next.config.ts`, `vercel.json`, `public/_headers`, or YAML configs) and you want `no-mistakes check` to fail whenever frontend code references an unlisted third-party network origin or a dynamic URL that cannot be verified statically.

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
      unanalyzable: report
```

## What it catches

For each indexable JS/TS/JSX/TSX file matched by the rule's `include`/`exclude` filters (excluding `options.cspFiles` themselves), `csp-allowed-origins` inspects network URLs (`http://`, `https://`, `ws://`, `wss://`, protocol-relative `//host`, and bracketed IPv6 literals such as `http://[::1]:3000` or `https://[2001:db8::1]`) in:

- `<script src="...">` checked against `script-src-elem` (falling back to `script-src`, then `default-src`)
- `<img src="...">` checked against `img-src`
- `<link href="...">` checked against `style-src-elem`/`style-src` (`rel="stylesheet"`), `img-src` (`rel` containing `icon`), `script-src-elem`/`script-src` (`rel="modulepreload"`), the matching `as` directive for `rel="preload"`, or `default-src` (skipping non-fetch hints such as `preconnect`, `dns-prefetch`, `canonical`, `alternate`)
- `<iframe src="...">` checked against `frame-src` (falling back to `child-src`, then `default-src`)
- `<audio src="...">`, `<video src="...">`, `<source src="...">` checked against `media-src`
- `fetch("...")`, `new WebSocket("...")`, and `new EventSource("...")` (including `window.`/`globalThis.`/`self.` qualifiers) checked against `connect-src`

Protocol-relative URLs (`//cdn.example.com/a.js`) are checked as `https://` origins. Template literals and string concatenations with a complete static origin prefix (such as `` `https://cdn.example.com/${path}` ``) are checked against that static origin, and conditional expressions with static branches check every branch. Relative paths (including guaranteed relative prefixes such as `` `/api/${id}` ``), `data:`, and `blob:` URLs pass cleanly.

Policy parsing follows browser CSP3 semantics:

- Within a single policy (including `[...].join("; ")` arrays, static or `'nonce-${...}'` / `'sha256-${...}'` template literals, and multi-line YAML block scalars), the **first** occurrence of a duplicate directive wins and later duplicates of the same directive are ignored.
- Across multiple policies (separate policy strings in a JS/TS module, multiple `Content-Security-Policy` headers, comma-separated serialized policies, or multiple files in `options.cspFiles`), policies are **intersected**: an origin is allowed only when every enforced policy allows it.
- Non-static JS/TS CSP expressions that cannot be statically resolved (such as `[...].join("; ")` with opaque dynamic elements or separators, `.filter(...).join(...)`, non-static `+` concatenations, non-nonce template interpolations, or conditional/logical policy branches) are never split into separate intersected policies; instead they are routed through `options.unanalyzable`.
- Unquoted non-JS CSP lines (such as YAML block scalars containing `img-src https://cdn.example.com;`) keep directive names intact and strip a `Header:` / key prefix only when the prefix before `:` has no whitespace, is not a URL scheme, and is not followed by `//`.

Dynamic URLs whose origin cannot be statically determined (for example `fetch(url)`, `` fetch(`https://${host}/x`) ``, `<img src={src} />`), malformed network URLs, dynamic `<link rel={rel}>` / `as={kind}` attributes, and unresolvable dynamic CSP policy expressions in `options.cspFiles` are routed through `options.unanalyzable` (default `report`).

## Options

- `cspFiles` (`string[]`, **required**): Non-empty list of repository-relative paths declaring static CSP directives. Supports JS/TS/JSX/TSX modules (read from static string/template literals, `+` concatenations, and `[...].join(...)` expressions in the prepared AST) as well as non-JS/TS files such as JSON, HTML, YAML, and `_headers`. Missing, unreadable, or directive-free files are reported as incomplete check inputs.
- `selfOrigins` (`string[]`, optional, default: `[]`): First-party origins (for example `https://app.example.com`) that satisfy `'self'` in the parsed CSP.
- `unanalyzable` (`"report" | "advise" | "ignore"`, optional, default: `"report"`): Controls how dynamic URLs, malformed network URLs, dynamic `<link>` `rel`/`as` attributes, and unresolvable dynamic CSP policy expressions are handled. `report` emits failing findings, `advise` emits non-failing advisories, and `ignore` omits unanalyzable diagnostics. Definite origin violations remain failing findings.

## Valid example

```ts
// src/csp.ts
export const contentSecurityPolicy = [
  "default-src 'self'",
  "img-src 'self' https://cdn.example.com https://*.assets.example.org https://[2001:db8::1]",
  "connect-src 'self' https://api.example.com http://[::1]:3000",
].join("; ");
```

```tsx
// src/page.tsx
export function Page(id: string) {
  void fetch(`https://api.example.com/v1/items/${id}`);
  void fetch("http://[::1]:3000/health");
  return <img src="//cdn.example.com/logo.png" alt="" />;
}
```

## Counterexample

```ts
// src/csp.ts
export const contentSecurityPolicy = [
  "default-src 'self'",
  "img-src 'self' https://cdn.example.com",
].join("; ");
```

```tsx
// src/page.tsx
export function Page(url: string) {
  // Reported: https://unlisted-cdn.example.net is not allowed by img-src
  // Reported (unanalyzable): dynamic fetch URL cannot be verified against CSP
  void fetch(url);
  return <img src="https://unlisted-cdn.example.net/hero.png" alt="" />;
}
```

## Fix

Add the third-party origin to the matching CSP directive in every enforced policy listed in `options.cspFiles`, or remove the third-party origin and serve the asset from `'self'`. For dynamic URLs, rewrite them to a static origin literal or static origin prefix (such as `` `https://api.example.com/${path}` ``), or set `options.unanalyzable` to `advise` or `ignore`:

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

The same configured rule runs through async `check()` and `analyzeProject()` check reports. `CspAllowedOriginsOptions` is exported from `no-mistakes` for rule configuration (`cspFiles`, `selfOrigins`, and `unanalyzable`).

## Related rules

- [`nextjs-redirect-destinations`](nextjs-redirect-destinations.md) — validate Next.js redirect destinations against configured routes and allowed hosts.
- [`config-path-references`](config-path-references.md) — validate static path references inside structured config files.
