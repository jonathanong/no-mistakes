# `no-mistakes symbols`

Print named exports and imports for TS/JS files.

```sh
no-mistakes symbols src/api.mts --include both --format json
no-mistakes symbols src/api.mts --mode signature-impact --symbol handler --format json
```

Use this before editing public APIs, replacing imports, or asking who consumes a
symbol with `dependents FILE#SYMBOL`.

Usage:

```sh
no-mistakes symbols <FILE>... [--root <PATH>] [--tsconfig <FILE>] [--config <FILE>]
  [--kind <KIND>]... [--include exports|imports|both]
  [--mode list|signature-impact] [--symbol <SYMBOL>]
```

Node callers can import `SymbolsMode` and `SymbolsInclude` from `no-mistakes`.

Key options: `--root`, `--tsconfig`, `--config`, repeatable `--kind`, `--include
exports|imports|both` (default `exports`), `--mode` (default `list`), `--symbol`,
`--format`, `--json`, and `--timings`. Omitted `--tsconfig` uses the config
that owns each file among the discovered configs; otherwise the root tsconfig.
Pass `--tsconfig <FILE>` only to force that one config.
Omitted `--format` is human when stdout is a terminal and JSON otherwise.

| Option | Default | Description |
| --- | --- | --- |
| `--config <FILE>` | auto-detected | Path to `.no-mistakes.yml` config. |

## Signature impact

Use signature-impact mode before changing a function export signature:

```sh
no-mistakes symbols src/api.mts --mode signature-impact --symbol handler --format json
```

This report groups the symbol definition, export/re-export paths, production
callers, test callers, and suggested focused tests. Signature-impact results
include `unanalyzed`, an array of `{ file, reason }` records. Unrelated failures
with retained source are reported there with an `unanalyzed-files` warning. Source text is decoded
lossily before analysis. A failure is an error when it could change the answer:
the target file, transitive importers of the target and their transitive
dependencies, or a raw occurrence of the requested symbol or a known export
alias in retained source. Imports or re-exports of these names through an opaque failed module
also fail conservatively, even when that module's link to the target is unknown.
Failures without retained source also fail conservatively: missing import edges
cannot establish independence when the file could not be read.
The default `symbols` output is unchanged unless `--mode signature-impact` is set.
Unrelated failures return exit 0; potentially relevant failures return exit 2.
Use JSON or YAML to inspect completeness before consuming `--format paths`.

Node API: `symbols(options)`.

## Language files

For a configured Swift, .NET, Python, Go, Rust, Rails, PHP, Java, Kotlin,
Elixir, or Dart file, `symbols` lists declarations from the same language facts
that build the canonical graph. Each export is `kind: "declaration"` with the
first source line that contains the name. Dotted names also match `\` and `::`
spellings, then the last name segment. `--include imports` lists recorded
import specifiers, not named bindings. `--kind` stays a TypeScript filter, so a
language file passed with `--kind` has no exports. `--mode signature-impact`
stays TypeScript and errors on a language file. Pass only language files, or
only TypeScript files, in one call. The file must sit in the configured
`tests.*.packages`, `modules`, `apps`, or `dotnet.projects` list.
`Package.swift` is not a symbol target. A language file outside that list errors.
