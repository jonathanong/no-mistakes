`invalid.ts` contains reachable imported and exported pass-throughs that remain
findings. `private-wrappers.ts` preserves the former invalid fixture verbatim as
a negative regression: its historical comment about meaningful exported names
describes the old, overly broad behavior, not the expected result today.

The former member-recall fixture is preserved verbatim at
`test-cases/eslint-snippets/fixture/ts-no-function-aliases/private-recall.ts`.
Those ambient/private targets are intentionally allowed. The release fixtures
beside it exercise reachable positives and the contract boundaries separately.

`legacy-v095-rule.cjs` under the snippets fixture directory is the verbatim rule
from `git show v0.95.0:packages/eslint-plugin-no-mistakes/src/rules/ts-no-function-aliases.js`.
The legacy compatibility tests execute that saved rule as the oracle for public
raw identifier calls. Expanded/private/asserted paths intentionally retain the
new conservative checks instead. `expanded-contracts.ts` separately protects
strict forwarding proofs, including parameter annotation presence.
