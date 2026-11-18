Forbid exact dependency-version assertions in tests (package manifests, parsed dependency maps, action refs, tool versions, release URLs/assets, tool logs, container images, setup-action runtime versions, Homebrew formulae, and runner labels).

`fixture/regex-action-refs` covers literal regex action refs, shape-only patterns,
placeholder versions, source lookalikes, suppression, and raw custom patterns.
`defaults/line-terminators.txt` deliberately stores malformed regex text for the
scanner unit regression without presenting it as a TypeScript module.
