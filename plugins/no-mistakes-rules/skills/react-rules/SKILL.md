---
name: react-rules
description: "MUST NOT / SHOULD NOT rules for React and Next.js: rendering, server and client boundaries, data fetching, caching, metadata, scripts, routing, and accessibility. Use when writing or reviewing React components or a Next.js app."
---

# React and Next.js rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- MUST NOT assume Next.js APIs from memory; check the installed version's docs.
- MUST NOT use `??` or `||` on a `ReactNode` where `null`, `false`, or `""` is a deliberate render; check explicitly.
- MUST NOT write `use(Promise.resolve(x))`; use the value or pass the existing promise.
- MUST NOT export `metadata` or `generateMetadata` outside route segment files.
- MUST NOT add raw `<script>` tags; use `next/script`.
- MUST NOT use dynamic `fetch` URLs or methods in pages; use literals so page-to-API links stay traceable.
- MUST NOT apply implicit caching (`force-cache`, cache directives) to request-time or authenticated routes.
- MUST NOT rely on a layout alone to make an authenticated page dynamic.
- SHOULD NOT scatter signed-out redirects across pages; centralize them in the layout.
- MUST NOT pass the current user or secrets across the server/client boundary as props.
- MUST NOT add API route handlers when endpoints belong to a separate backend.
- MUST NOT point redirects or rewrites at routes that do not exist.
- MUST NOT hand-build entity URLs; use the shared link helpers.
- MUST NOT bake browser config into the build; load it at runtime.
- MUST NOT render user-supplied image URLs directly; proxy them.
- MUST NOT nest interactive elements inside a clickable row or link.
- MUST NOT disable accessibility checks to make stories or tests pass.
- SHOULD NOT use IIFEs in JSX; extract a component or helper.
- SHOULD NOT add identity-only memoization when the React Compiler is on.
- SHOULD NOT import server-only modules into stories or client components.
