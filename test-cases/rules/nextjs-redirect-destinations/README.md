Check that Next.js redirect and rewrite destinations resolve to App Router pages.

`tracked-routes` is copied into a synthetic Git repository by native unit and CLI
tests. Only its tracked/group/dynamic/catch-all pages are staged initially;
`untracked` remains visible and `ignored` is excluded by `.gitignore.fixture`.
The tests stage both missing pages afterward to prove the findings clear.
