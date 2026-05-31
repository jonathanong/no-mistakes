# `no-mistakes/nextjs-no-manual-script-tags`

Prefers `next/script` over raw JSX `<script>` tags.

Why: Next.js script loading behavior is explicit and analyzable through
`next/script`.

Counterexample: `<script src="/analytics.js" />` in JSX.

Fix: replace raw script tags with `Script` from `next/script`.
