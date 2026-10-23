`invalid.ts` contains reachable imported and exported pass-throughs that remain
findings. `private-wrappers.ts` preserves the former invalid fixture verbatim as
a negative regression: its historical comment about meaningful exported names
describes the old, overly broad behavior, not the expected result today.

The former member-recall fixture is preserved verbatim at
`test-cases/eslint-snippets/fixture/ts-no-function-aliases/private-recall.ts`.
Those ambient/private targets are intentionally allowed. The release fixtures
beside it exercise reachable positives and the contract boundaries separately.
