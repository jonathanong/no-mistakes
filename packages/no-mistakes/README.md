# no-mistakes

Unified static codebase intelligence CLI for TS/JS module graphs, symbols,
React traits, queue hops, server routes, and project checks.

```bash
npm install --save-dev no-mistakes
npx no-mistakes dependencies src/main.mts --json
npx no-mistakes dependents src/utils.mts --json
npx no-mistakes symbols src/utils.mts --json
npx no-mistakes check --json
```

External `no-mistakes-*` executables on `PATH` can be invoked as subcommands.
For example, after installing `no-mistakes-scripts`:

```bash
npm install --save-dev no-mistakes-scripts
npx no-mistakes rust-no-inline-tests crates/*/src
npx no-mistakes rust-max-lines-per-file crates/*/src crates/*/tests
```

See the full documentation in [docs/](../../docs/README.md) and the
[CLI reference](../../docs/cli-reference.md).
