<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

### Imported and literal-identifier helpers

The extractor lowers each top-level helper a file declares into a small model of
its body once, during that file's single AST pass, and records every call to
an imported helper as a hole in the caller's SQL. The request's prepared
resolver then fills the holes by running the helper's model against that call's
arguments. Nothing is parsed or read a second time, and a call whose link
cannot be proven stays `Dynamic` exactly as before.

Three fixed helper shapes recover directly:

- **Fixed text**: a helper returning a string, template or `sql` fragment that
  does not depend on its arguments. Any arguments are ignored.
- **Literal identifiers**: a helper whose one `return` is an _untagged_
  template literal whose every `${...}` is one of the helper's own parameters,
  for example `` (col: string) => `(${col} IS NULL)` ``. A call that passes a
  string literal for **every** parameter has the literals spliced in as SQL
  text, so `ownerVisible('l.parent_id')` contributes `(l.parent_id IS NULL)`.
  This works for same-file helpers too. A string literal is a quoted string
  (`'l.parent_id'` or `"l.parent_id"`). A no-substitution template literal
  (`` `l.parent_id` ``) is not one, on either path, so it is not spliced and
  that call stays `Dynamic`. A helper whose arguments never reach the SQL
  still accepts the template as a known non-text argument. A non-literal, missing, extra or
  spread argument, a default, rest, optional or destructured parameter, a
  parameter used anywhere but an interpolation of that template (including
  `col.length`, `col + '...'` or a local alias), a literal containing a
  newline, or an async or generator helper keeps the call `Dynamic`. Tagged
  `sql` templates are different: their `${...}` values are bound parameters,
  not SQL text.
- **Statement builders**: a helper that appends fixed `sql` fragments to a
  `SQLStatement` parameter and returns it. The caller supplies a trusted tagged
  template or a fully recovered local builder, as for same-file helpers.

Imports are resolved with the same resolver as the rest of the request, so
these forms work:

- relative specifiers with explicit extensions (`./visibility-sql.mts`);
- `tsconfig` `paths` aliases such as `@modules/feed-query-builders`;
- pnpm workspace package names (`@scope/package`, and its `exports` subpaths)
  declared in the workspace map, with or without installed `node_modules`;
- barrels. `export * from`, `export { name as alias } from`, `export { default
  as name } from` and an imported name that is re-exported are followed for up
  to **8 re-export hops** from the module the caller imports. An explicit
  re-export outranks `export *`; two `export *` sources that declare the same
  name are ambiguous unless they reach one declaration.

Helpers also compose with the rest of a builder:

- **Conditional alternatives.** A hole after (or inside) a conditional append is
  filled in every complete alternative, so the 16-version cap applies as it
  always did; past the cap the call stays `Dynamic`.
- **Statement-level builder helpers.** `appendOrdering(query);` on an imported
  `SQLStatement`-parameter helper appends that helper's fixed fragments to
  `query`, exactly like `query.append(...)`. The argument must be a tracked
  local builder; the other arguments must be literals or plain identifiers.
- **IIFE-local builders.** ``const built = (() => { const query = sql`...`; ...;
  return query; })();`` gives `built` the final state of `query`, including its
  finite alternatives and any holes. The function must be a directly invoked,
  non-async, non-generator arrow or function expression with no name,
  parameters or arguments, must not use `this` or `arguments`, must declare the
  returned local with `const` or `let` and must have exactly one `return`, last.
  The outer result binding (`built` in this example) must use `const`.

Imports resolve through the same per-importing-file scoped resolver as the
rest of the request, so a package's own `paths` win over the root's `@/*`.

#### Helper control flow and nested composition

A helper that is not one of the fixed shapes above is run, not summarized. Its
body is lowered to a small model: `const`/`let` locals, `if`/`else`, `return`,
`throw`, a `switch` whose arms each return a literal statement or literal
identifier and whose `default` is proven to throw, `for (const x of ...)`,
`.append(...)` chains, tagged `sql` templates, untagged templates, string
literals, ternaries, `&&`/`||`/`??`/`!`, `===`/`!==`/`==`/`!=`, `+`, object
and array literals, member access (including `?.`) and calls to other
top-level helpers. Anything else lowers to an opaque node that fails the run
only when a path actually reaches it.

Each call runs the model against the call site's arguments:

- **Bounded alternatives.** A test over known values (a literal, a literal
  object property, an omitted argument, `undefined`) is decided. A test over an
  unknown scalar forks into both outcomes, and a repeated test of the same
  parameters agrees with its first answer. `throw` ends a path, since the
  executor never runs after it, and an early `return` ends its path with the
  SQL appended so far. An `if` that only throws, and whose test cannot write
  SQL (`!Number.isInteger(limit)`, or `&&` / `||` of such member calls, not a
  helper call or `.append`), is skipped. A recoverable `switch` yields one
  alternative per arm. An unknown selector, or a name that is not bound,
  keeps every arm; a selector that matches none reaches `default` and that
  path ends. `default` is proven only when the analyzer sees `throw`, or a
  same-file callee whose body always throws — a function named `assertNever`
  is not enough. A `switch` that is not this shape, including a `default` that
  does not throw, stays `Dynamic` rather than keeping the known arms. At most
  **16** paths survive; past that the call stays `Dynamic` and the union is
  not truncated.
- **`??`.** A bound scalar `??` fallback stays a bind, so a string constant
  there is not its own SQL alternative. `??` on a name the helper does not
  bind (it may be an SQL fragment) stays unknown and does not publish the
  fallback. A fallback that is itself a builder abstains.
- **Bind values.** `${options.limit}` inside a tagged template is a bind, so it
  needs no literal. An object-literal argument is accepted when its properties
  are literals, identifiers or member chains; the helper may read it only in a
  bind position or a branch test.
- **Literal text.** Raw text, such as ``.append(`${alias}.approved_at IS NOT
  NULL`)``, an object property passed as `.append(config.cteAlias)`, or a
  string parameter appended directly, needs a string literal at the call site
  (without a newline). A non-literal in an identifier position fails the call.
- **Defaults.** An omitted argument and an explicit `undefined` both take the
  parameter's default.
- **Helpers calling helpers.** A call resolves through the helper's own module
  (its imports and its top-level functions) and is memoized per `(executor,
  module, helper, arguments)`. Composition is bounded to **8** helpers deep, and
  a cycle fails closed. A call to a function with no SQL of its own (for
  example an `assertSafeAlias(alias)` guard that only throws) is ignored when it
  is a statement and receives only plain values.
- **Builders.** A parameter annotated with the `SQLStatement` type receives the
  caller's builder. The helper may append to it, alias it, pass it to another
  helper and return it, but never read it back. A value-position call
  (`read(helper(query))`) must return the very builder it was given.
- **Bounded loops.** `for (const x of list)` is unrolled when the call site
  passes an array literal of at most 16 known elements (or the parameter's
  default is one). A loop over anything else fails the call.
- **Spreads.** In an object literal, a spread of an unknown or open value makes
  every property written before it unknown; properties written after it stay
  known. With `{ offset: false, ...opts }`, `o.offset` is unknown, so both
  branches are checked.
- **Nested functions.** A helper whose body declares any nested function, even
  inside a block, is not run and the call stays `Dynamic`. This covers local
  helpers hoisted above their declaration and locals that shadow a
  module-level helper.
- **Duplicate builders.** A nested helper call that passes the same builder to
  two `SQLStatement` parameters stays `Dynamic`.

Calls recover in the same places a fixed helper does: as an `.append(...)`
argument, as a statement that extends a builder, as an executor argument, bound
to a `const` and appended later (including inside a branch), and as the first
link of a fluent chain.

#### Executor templates over a builder's `.text`

``read(`/* name */ WITH ${builder.text}`, builder.values)`` is recovered when
the template interpolates exactly one `.text` of a plain identifier whose
builder is fully recovered (including finite alternatives) and the second
argument is that same identifier's `.values`. The builder's SQL replaces the
interpolation and the other pieces stay static text. A different second
argument, a third argument, any other interpolation, a `.text` of something that
is not a tracked builder, or a builder that is only partly recovered keeps the
call `Dynamic`.

#### Fluent statement chains

`query.append(a).append(b)` as a statement appends every link to the tracked
builder `query` in source order, so a later link is part of the recovered SQL.
A chain whose root is not a tracked local builder recovers nothing. A tracked
builder or recovered helper result appended into a fluent expression is copied
at that moment, as the library does.

#### What stays `Dynamic`

Everything else fails closed and stays `Dynamic`: an unresolved or external module,
a module outside the checked files, a workspace package that is not declared, a
re-export chain past 8 hops, a re-export cycle, ambiguous `export *` names,
`export * as ns`, namespace-import member calls (`ns.helper()`), CommonJS
`require`, dynamic `import()`, a local variable shadowing the imported name or
the SQL tag, any helper the walk could not lower, a path through `while`, `do`,
`for`, `for...in`, a `switch` without a proven throwing `default`, `try`,
assignment, a closure or a spread of a
non-object, more than 16 alternatives, helpers more than 8 deep, a cycle, a
caller's builder handed to a parameter that is not annotated `SQLStatement` or
to a function whose body is unknown, a helper that reads the builder it was
given, and an IIFE outside the exact shape above (parameters, arguments,
`async`, generators, `this`, `arguments`, several returns, returning an outer or
captured binding, statements after the return, a named function expression, or
an outer result bound with `let`). A same-file helper that receives the caller's builder is
composed only when the call is straight-line in the builder's own function; in
a conditional, loop or nested function it keeps the established per-call
effect analysis. A call that stays `Dynamic` keeps only the verified leading
statement before the first hole, as before.

#### Helper-line attribution and suppression

Recovered SQL keeps the helper's physical lines. A quoted string spliced into
a `${param}` is the exception: those bytes are credited to the literal's own
line and file. `tail('OFFSET 40')` reports `postgres-no-offset` on that
literal, while an `OFFSET` the helper itself wrote stays on the helper's
line. Findings from every rule that
consumes embedded statement facts (all of the `postgres-*` DML rules, including
`postgres-required-predicates`, `postgres-explicit-columns`,
`postgres-no-offset`, `postgres-bounded-statements`, `postgres-sql-shape-policy`
and `postgres-sql-statement-policy`) are reported in the helper's module at the
helper's line, so a directive written there applies. A directive in the calling
file also suppresses the finding: file-level `no-mistakes-disable-file`, a
directive on the executor call, or one on the imported-helper call. In that
case the finding stays at the caller's location, so suppression accounting
(`--include-suppressed`) is unchanged. Each `(module, export)` pair is located
once per request, however many calls import it.

[PostgreSQL fact sources](../postgres-facts.md)
