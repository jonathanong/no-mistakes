---
name: react-rules
description: "Load before writing or reviewing React components, hooks, or a Next.js page, layout, or route handler. Lists the defects to avoid and the patterns that need a stated reason: rendering, server and client boundaries, caching, metadata, scripts, routing, accessibility."
---

# React and Next.js rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- MUST NOT assume Next.js APIs from memory; check the installed version's docs.
- MUST NOT use `??` or `||` on a `ReactNode` where `null`, `false`, or `""` is a deliberate render; check explicitly.
- MUST NOT write `use(Promise.resolve(x))`; use the value or pass the existing promise.
- MUST NOT export `metadata` or `generateMetadata` outside route segment files.
- MUST NOT add raw `<script>` tags; use `next/script`.
- MUST NOT apply implicit caching (`force-cache`, cache directives) to request-time or authenticated routes.
- MUST NOT rely on a layout alone to make an authenticated page dynamic.
- MUST NOT pass the current user or secrets across the server/client boundary as props.
- MUST NOT point redirects or rewrites at routes that do not exist.
- SHOULD NOT render user-supplied image URLs directly; the viewer's browser fetches them and leaks the viewer's IP address to that host.
- MUST NOT nest interactive elements inside a clickable row or link.
- MUST NOT disable accessibility checks to make stories or tests pass.
- SHOULD NOT add identity-only memoization when the React Compiler is on.
- SHOULD NOT import server-only modules into stories or client components.
