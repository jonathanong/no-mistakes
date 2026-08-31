# planning-impact

`planning-impact` is an npm-package integration command. It is not available
from the Cargo-installed native `no-mistakes` binary.

```sh
npx no-mistakes planning-impact \
  --changed-files /private/run/changed-files.txt \
  --output-dir /private/run \
  --profile ci
```

The current working directory is the analysis root. The command performs one
prepared `analyzeProject()` request and atomically publishes `dependencies`,
`dependents`, `symbols`, and Vitest `plan` artifacts into the private output
directory. The manifest and output-directory privacy and placement contract is
the same as [`writePlanningImpactArtifacts`](../node-api.md#node-n-api-guide).

Options:

- `--changed-files <manifest>` and `--output-dir <directory>` are required.
- `--broad` opts into broader test-plan behavior.
- `--timeout <seconds>`, `--lock-timeout <seconds>`, `--fail-on-lock`,
  `--jobs <count>`, and `--profile ci` use the matching Node API controls.

Successful execution is silent. Argument or analysis errors write a
UTF-8-safe diagnostic of at most 4 KiB to stderr and exit `1`.
