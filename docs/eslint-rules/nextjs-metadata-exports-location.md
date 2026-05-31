# `no-mistakes/nextjs-metadata-exports-location`

Restricts Next.js metadata exports to route segment files.

Why: placing metadata in segment files keeps Next.js behavior and static analysis
aligned.

Counterexample: exporting `metadata` from a shared component file.

Fix: move metadata exports to the owning `page`, `layout`, or supported route
segment file.
