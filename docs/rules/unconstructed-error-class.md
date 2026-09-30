# `unconstructed-error-class`

Reports exported error classes that no production code ever constructs or
subclasses.

## Why and when

An error class that nothing throws is dead error handling. The `catch` block, the
`instanceof` check, and the type guard that mention it look like a real failure
path, but they can never run. The class usually survives a refactor that removed
its only `throw`, and neither an unused-export check (its `instanceof` uses count
as references) nor a text search flags it.

Enable this rule when a repository keeps typed error classes and wants dead ones
removed. It is opt-in: it does nothing until it appears under `rules:`.

## What it catches

An exported class is an error class when it extends `Error`, a built-in error
subclass (`TypeError`, `RangeError`, `SyntaxError`, and the other standard error
constructors, also spelled `globalThis.Error` or `window.Error`), or another
class in the analyzed project that is itself an error class. Chains of any
length are followed, through imports, aliases, barrels, `export *`, namespace
imports, and workspace package specifiers.

An error class is reported unless non-test source does one of these:

- constructs it with `new`, from any file that can reach it, including through
  an alias or a re-export;
- subclasses it with `extends`.

`instanceof`, a type guard, a `catch` clause, and a type-only reference are not
construction. Test files never count as construction sites, so a class that only
a test builds is reported. Test files are `__tests__` directories and
`.test.*` / `.spec.*` files; the `testFiles` option adds more.

A `new this()` or `new` of an unresolved value inside a class member counts as
constructing that class, so static factories such as
`static create() { return new this(); }` keep their class alive.

Declaration files (`.d.ts`, `.d.mts`, `.d.cts`) are skipped: they describe code
outside the analyzed source, so their classes are never reported.

## Options

```yaml
rules:
  - rule: unconstructed-error-class
    scope: repository
    include: ["backend/**"]
    exclude: ["**/generated/**"]
    message: Remove or start throwing this error class
    options:
      testFiles:
        - "test-helpers/**"
        - "**/test-helpers/**"
```

`testFiles` is a list of repository-relative globs for files that only exercise
code, such as a shared `test-helpers/` directory. A construction there does not
count. It defaults to an empty list, so only the built-in `__tests__`,
`.test.*`, and `.spec.*` classification applies. Unknown options are rejected.

The common rule fields apply to the declaration only. `include`, `exclude`, and
`projects` choose which error classes are reported; a construction in a file
outside them still counts, because the rule must see every use to know a class is
dead. `message` is prefixed to the class name: `message: Dead error` reports
"Dead error: `ClassName`". Without it the message names the class and says it is
never constructed or subclassed in non-test source.

The rule builds the repository call graph, so give it `scope: repository`.

## Valid example

Compliant example: every exported error class is thrown somewhere.

```ts
// errors.ts
export class NotFoundError extends Error {}

// handler.ts
import { NotFoundError } from "./errors.js";

export function load(id: string) {
  throw new NotFoundError(id);
}

// route.ts
import { NotFoundError } from "./errors.js";

export function respond(error: unknown) {
  return error instanceof NotFoundError ? 404 : 500;
}
```

An abstract or shared base is satisfied when a subclass exists:

```ts
export abstract class AppError extends Error {}
export class ValidationError extends AppError {}
```

`AppError` is used because `ValidationError` extends it. `ValidationError` is
reported until something constructs it.

## Counterexample

```ts
// guard.ts
export class TopicSearchError extends Error {}

export function isTopicSearchError(error: unknown) {
  return error instanceof TopicSearchError;
}
```

Nothing constructs `TopicSearchError`, so `isTopicSearchError` can never
return true. The finding is reported at the `class` line.

## Fix

Either delete the class together with the `instanceof` checks and type guards
that mention it, or add the `throw new ...` that makes it real. Removing a
subclass can leave its base unconstructed and newly reported, so rerun the check
until it is clean.

## Suppression

The finding points at the class declaration. Suppress a class that is built in
a way the graph cannot see, for example by a plugin loader or a dynamic lookup:

```ts
// no-mistakes-disable-next-line unconstructed-error-class: built by the plugin loader
export class PluginError extends Error {}
```

`no-mistakes-disable-line` on the declaration line and
`no-mistakes-disable-file unconstructed-error-class` at the top of a file (every
class in it) work too.

## Limitations

The rule reads static code only.

- Dynamic construction (`new (registry[name])()`, `Reflect.construct`) is not
  seen, so such a class is reported; suppress it.
- A base class imported from an external package (`TRPCError`, `HttpError`) is
  not an error root, because the package source is not analyzed. Classes that
  extend one are not reported.
- Mixin and expression bases (`extends withCode(Error)`) and class expressions
  are not tracked.
- A dead subclass still satisfies its base. The check converges over repeated
  runs as you delete dead classes.
- Only exported classes are reported. A non-exported class that nothing uses is
  a plain unused declaration.
- `export declare class X extends Error {}` in a regular `.ts` file is treated
  like a real class, because the `declare` modifier is not tracked. Only
  declaration files are skipped; move the declaration there or suppress it.

## Related rules

[`unique-exports`](unique-exports.md) keeps public export names unambiguous;
`unconstructed-error-class` covers exported names that can never be produced.
[`forbidden-calls`](forbidden-calls.md) uses the same call graph to ban
invocations rather than require them.
