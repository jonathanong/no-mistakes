# eslint-plugin-no-mistakes

ESLint and Oxlint rules that keep TS/JS, React, Next.js, and Playwright code
static enough for `no-mistakes` analyzers. They exist so agents cannot hide
callers, fetches, or selectors behind aliases and dynamic values. See
[why no-mistakes exists](../../docs/why.md).

```sh
npm install --save-dev eslint-plugin-no-mistakes
```

For `async-call-disposition` to inspect imported callback implementations when
the linter does not provide a TypeScript program (including Oxlint's
`--type-aware` mode), also install `typescript` and
`@typescript-eslint/parser` in your project:

```sh
npm install --save-dev typescript @typescript-eslint/parser
```

These are optional peer dependencies. This fallback requires TypeScript 5 or
later for Bundler module resolution. With older TypeScript or either peer
unavailable, calls that rely on this cross-file proof are reported as unproven.
Same-file proofs and explicitly configured awaiting-consumer
contracts continue to work. When ESLint provides a typed TypeScript program,
the rule continues to use that program for its typed proof.

Configure the plugin in a flat config and scope rules with `files` globs. For
example, this checks test files for skipped or exclusive tests while allowing
`todo`:

```js
import noMistakes from "eslint-plugin-no-mistakes";

export default [
  {
    files: ["**/*.test.*", "tests/**"],
    plugins: { "no-mistakes": noMistakes },
    rules: { "no-mistakes/test-no-skips": ["error", { allow: ["todo"] }] },
  },
];
```

See [`test-no-skips`](../../docs/eslint-rules/test-no-skips.md) for the rule's
behavior and configuration, and the [ESLint rule index](../../docs/eslint-rules/README.md)
for the complete list.

To cap long Vitest timeouts, enable `no-mistakes/vitest-timeout-cap` in the
test and Vitest config files selected by your flat-config globs. It defaults to
5,000 ms for configured test and hook timeouts and 30,000 ms for per-test and
hook overrides. See the [rule documentation](../../docs/eslint-rules/vitest-timeout-cap.md)
for options and examples.

`gh-api-argument-safety` is an opt-in rule for structured endpoint fields and explicit API methods.
See [options and examples](../../docs/eslint-rules/gh-api-argument-safety.md).

`no-argv-main-guard` is an opt-in rule for symlink-sensitive entrypoint comparisons.
See [options and examples](../../docs/eslint-rules/no-argv-main-guard.md).

`no-comment-phrases` is an opt-in rule for configured phrases in line and block comments.
See [options and examples](../../docs/eslint-rules/no-comment-phrases.md).

`forbidden-member-chains` is an opt-in rule for configured static member-access chains.
See [options and examples](../../docs/eslint-rules/forbidden-member-chains.md).

`no-object-mutation` is an opt-in rule for shared object writes and mutating calls.
See [options and examples](../../docs/eslint-rules/no-object-mutation.md).

`postgres-no-async-sql-builder` is an opt-in rule for async SQL builders that never wait.
It suggests removing `async` only when the visible contract allows a synchronous return, and it never autofixes.
See [options and examples](../../docs/eslint-rules/postgres-no-async-sql-builder.md).

`require-options-on-imported-call` requires a static options object on configured
imports, globals with no local binding, and terminal methods with a matching receiver shape.
See [options and examples](../../docs/eslint-rules/require-options-on-imported-call.md).

`restricted-api-usage` is an opt-in import/export-name and call-name policy.
See [options and examples](../../docs/eslint-rules/restricted-api-usage.md).

Opt-in client contracts: [`no-consecutive-unbatched-reads`](../../docs/eslint-rules/no-consecutive-unbatched-reads.md) configures adjacent awaited read batching; [`require-top-level-registration`](../../docs/eslint-rules/require-top-level-registration.md) configures owner-module lifecycle registration. Both support ESLint/Oxlint and have no product defaults.
