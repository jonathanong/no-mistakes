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

A class declared with `declare` (`export declare class X extends Error {}`) or
inside an ambient block (`declare namespace N { ... }`, `declare module "x" { ... }`,
`declare global { ... }`) is never reported, even in a regular `.ts` file: it
describes code outside the analyzed source like a declaration file does.

A class declared inside a TypeScript `namespace` is reported like any other class
when nothing constructs or subclasses it. The graph resolves these references to
the member, nested and dotted namespaces included:

- `new Errors.TopicError()`, where `Errors` is declared in the same file or is an
  exported namespace imported from another module, directly, under an alias
  (`import { Errors as E }`), under a renamed export (`export { Errors as
default }`), or through barrels and `export *`;
- a bare `new TopicError()` written inside the body of the namespace that
  declares `TopicError`, including inside a nested namespace or function there;
- `new Outer.Inner.DeepError()`, and `new A.B.C()` for `namespace A.B { export
class C extends Error {} }`;
- `class Child extends Errors.Base {}`, which subclasses `Errors.Base`.

The rule stays quiet about every class of a namespace when any use of that
namespace is not a static member access the graph can follow, because that use
might build any of its classes:

- an alias or destructuring (`const E = Errors`), passing the namespace or one of
  its members as a value (`register(Errors)`, `register(Errors.TopicError)`), or
  a computed access (`new Errors[name]()`);
- a member handed on through `bind`, `call`, or `apply`
  (`new (Errors.TopicError.bind(null))()`), which can build the class somewhere
  the graph does not see. The list is deliberately these three: any other method
  on a member, such as a static guard (`Errors.TopicError.is(value)`), only reads
  the class, so it neither keeps the class quiet nor builds it;
- `export default Errors`, `export = Errors`, or `import Alias = Errors.Inner`;
- a namespace merged with a class, function, variable, enum, or import of the
  same name, whose statics or members the graph cannot tell apart from the
  namespace's classes. Several `namespace` blocks of one name are not a merge of
  this kind: they share one member table, and each block's classes are reported
  like any other;
- a module imported as a whole (`import * as errors`, `import("./errors")`,
  `require`, `import x = require()`) when it exports or re-exports the namespace.
  A module read whole exposes only the namespaces it exports: a barrel that
  re-exports `Exposed` by name leaves a namespace it does not re-export
  reported, while `export *` or an export the graph cannot follow exposes every
  namespace of that module;
- a construction that names a member the namespace does not declare
  (`new Errors.Missing()`, or `new Errors.Missing.Factory()` when `Errors` has no
  `Missing` namespace);
- a namespace in a global script file, one with no `import` or `export`, which
  any other file can reach.

Several things are not uses of a namespace. A parameter or local that shadows the
namespace or one of its classes (`function f(Errors) { return new Errors.X(); }`)
names that binding, so the construction builds no class of the namespace and does
not keep it quiet. A value that only shares the name, such as a top-level
`const Inner = {}` beside `namespace Errors { export namespace Inner { ... } }`,
is no use of the nested namespace: a name counts as a use only when it resolves
to a declared namespace path or an imported binding. A namespace body is a scope
of its own, like a function body: a `const`, `let`, or hoisted `var` in it hides
a name inside that body only, so `namespace Helpers { const Errors = {}; }` leaves
a later `new Errors.X()` naming the imported namespace, and a class and a nested
namespace of one name in one body are a single merged value that hides nothing.
A bare decorator (`@Errors.mark`) calls the member it names and is not a use of
the namespace. A name written only in an erased type (`typeof Errors`,
`implements Errors.Marker`, an interface that extends `Errors.Base`) is not a use
of the namespace, because it runs no code. `import type x = require("./m")` is erased at compile time and uses no module. A
sourced clause (`export { Errors } from "./m"`) exports the `Errors` of `./m`,
never a namespace of the same name declared locally.

These uses count wherever they are written, test files included: a test file that
copies a namespace into a variable keeps its classes quiet, even though a test
that constructs a class never keeps that class alive.

Only a class another module can reach is exported: an `export class` inside a
namespace that its module exports, at every level of nesting. A class without
`export`, or in a namespace its module does not export, is never reported. Two
classes that share a name in different namespaces of one file share a scope key,
so a use of either silences both.

Such a class still counts as an error class and still counts as a use of its
base, so a subclass elsewhere keeps its base alive.

## Files that fail to parse

When a non-test source file fails to parse or read, the rule stops with an error
naming that file (and how many others failed too). A construction in a file the
rule cannot see would go unnoticed, so reporting a class as unconstructed would
risk a false finding. This differs on purpose from
[`forbidden-calls`](forbidden-calls.md#unknown-calls-and-suppression), which
ignores unrelated files that fail to parse: a file it cannot read can only hide a
finding there, never add one.

Fix the file, or, if it really is test-only, classify it as a test file with
`testFiles`. Test files that fail to parse do not stop the rule, because they
never count as construction. Declaration files that fail to parse do not stop it
either: they hold no construction.

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

## How it works

The rule reads the same canonical call graph as `forbidden-calls`; it has no
resolver of its own. A `new` expression is a resolved `construct` call site, so
an alias, a barrel, a namespace import, and a workspace package name (`@scope/pkg`)
all reach the class the way they do for `forbidden-calls`. An `extends` clause
is an opt-in `extends` edge from the subclass to its base class, plus a
declaration record with the class line and export state
([graph edges](../graph-edges.md)). The base of an `extends` clause is not a
call, so it never shows up in `forbidden-calls` and never counts as a
construction. The rule follows `extends` edges to a built-in error, and treats a
subclass in non-test source as a use of its base.

A class inside a `namespace` is resolved the same way, through the namespace's
member path: the extractor records each namespace member, and each namespace
used as a value, in the one parse it already does per file. The graph then looks
`Errors.Inner.X` up in the namespace the name reaches, within the file or through
imports and re-exports, and credits the class by its exact declaration.

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

The rule reads static code only. Most limitations below hide a dead class; the
first reports a class that is not really dead, which a suppression comment
resolves.

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
- A class declared with `declare` or inside an ambient block is not reported, and
  neither is a namespaced class whose namespace is used in a way the graph cannot
  follow (see [What it catches](#what-it-catches)).
- An exported alias of a class is not recognized as an export, so the class is
  not reported. With `const PublicError = InternalError;` and
  `export { PublicError };`, `InternalError` is skipped.
- An alias of a built-in error used as a base is not followed, so the class is
  not reported. With `const BaseError = Error;`, a class that extends
  `BaseError` is skipped.
- An unresolved `new`, such as `new this()`, credits the outermost class of the
  member it sits in. A class nested inside an error class's method can
  therefore hide that error class.

## Related rules

[`unique-exports`](unique-exports.md) keeps public export names unambiguous;
`unconstructed-error-class` covers exported names that can never be produced.
[`forbidden-calls`](forbidden-calls.md) uses the same call graph to ban
invocations rather than require them.
