---
name: react-rules
description: "Load before writing or reviewing React components, hooks, or a Next.js page, layout, route handler, redirect, or rewrite. Lists the defects to avoid and the patterns that need a stated reason: rendering, server and client boundaries, caching, metadata, scripts, routing, accessibility."
---

# React and Next.js rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- SHOULD verify unfamiliar Next.js APIs against the installed version's docs.
- MUST NOT use `??` or `||` on a `ReactNode` where `null`, `false`, or `""` is a deliberate render; check explicitly.
- MUST NOT write `use(Promise.resolve(x))`; use the value or pass the existing promise.
- MUST NOT export `metadata` or `generateMetadata` outside route segment files.
- SHOULD NOT use raw `<script>` tags for executable code in Next.js without a loading or execution-order requirement that `next/script` cannot meet.
- MUST NOT apply implicit caching (`force-cache`, cache directives) to request-time or authenticated routes.
- MUST NOT rely on a layout alone to make an authenticated page dynamic.
- MUST NOT expose server secrets through client props.
- SHOULD NOT pass user fields the client does not need.
- MUST NOT point redirects or rewrites at routes that do not exist.
- SHOULD NOT render user-supplied image URLs directly; the viewer's browser fetches them and leaks the viewer's IP address to that host.
- MUST NOT nest interactive elements inside a clickable row or link. The opt-in `react-no-nested-buttons-and-links` check enforces native HTML button and link ancestry in JSX.
- MUST NOT treat an empty or short page as the end of a paginated list when the API returns a next-page flag or cursor; continue until it says there is no next page. Show the empty state only when there are no results and no next page.
- SHOULD NOT disable accessibility checks to make stories or tests pass without a scoped exception and a stated reason.
- SHOULD NOT add identity-only memoization when the React Compiler is on.
- SHOULD NOT import server-only modules into stories or client components.
- SHOULD NOT use implicit locales in `toLocaleString` or `Intl` formatting when server rendering and hydration must produce identical text. Use an explicit locale to avoid hydration mismatches. Browser-only formatting after hydration can use the browser locale.
