# Low-effort query annotations

`effort: max` is the default and preserves helper, branch, builder, and imported
value inference. At effective `effort: low`, this rule does not request or run
the annotation evaluator. It checks literal heads directly, through one local
const, or through mutable literal bindings as described below:

```ts
// The leading named comment is literal at the executor call.
query(`/* posts/list */ SELECT id FROM posts WHERE id = ${id}`);
query("/* posts/list */ " + body);

// Or one local const has that literal head and is passed directly.
const statement = `/* posts/list */ ${body}`;
query(statement);
```

Any tagged template qualifies in either position when its first quasi starts
with the complete named comment. Tag identity and import form do not matter;
the tag can be a named, default, namespace, or renamed import, or another tag
expression:

```ts
import sql from "@example/sql";
import * as templates from "@example/templates";
query(sql`/* posts/list */ SELECT id FROM posts WHERE id = ${id}`);
const statement = templates.sql`/* posts/list */ SELECT id FROM posts WHERE id = ${id}`;
query(statement); // A single local const passed directly also qualifies.
```

At `low`, `trustedSqlTags` is deprecated for this rule and ignored; configuring
it produces a configuration warning. This does not change its behavior for
`max` analysis or for other SQL rules that consume the option.

A tagged template or string keeps its literal head through static `.append(...)`
chains of any depth. Each append must have exactly one string literal or tagged
template argument; a tagged interpolation is a bind placeholder, so its value
does not need inference. The annotation fast path checks syntax and the original
head without composing suffix text:

```ts
query(sql`/* posts/list */ SELECT id FROM posts`.append(sql` WHERE id = ${id}`));
const statement = sql`/* posts/list */ SELECT id FROM posts`;
statement.append(sql` WHERE id = ${id}`);
query(statement);
```

A local `const` initialized by the exact `sql-template-strings` default or
named `sql` tag can pass its `.text` to an executor with its `.values` as the
second argument. The leading name is proven from that trusted builder and
survives safe appends. Both fields must be direct, non-optional reads; a
computed field, escaped or overwritten builder, arbitrary object with a `text`
field, or caller-supplied SQL remains unresolved.

Low also follows a same-file function declaration or `const` arrow with one
returned expression and a fixed named literal head. A block body must have
one final top-level return; a concise arrow body is the returned expression.
Tagged returns must use the `sql-template-strings` default or
named `sql` import. A direct trusted tagged return can append an unknown suffix;
the fixed leading comment remains visible. An opaque append through another
tag does not establish a safe prefix or make later reads of that builder safe.
Async and generator helpers, unproven
return paths, reassigned or shadowed callees, and optional or spread helper
calls remain unresolved. A directly imported helper from another prepared
source file also qualifies when that file directly exports the same fixed
return summary, including a one-hop local export alias. The request's resolver checks the exact import and export;
For direct helper imports, re-exports and unavailable package implementations
remain unresolved. A helper
may also return one local `const` initialized from the trusted default SQL tag
when that binding has only prefix-preserving `.append(...)` uses before the
single final return. Aliases, captures, property writes, and passing the
builder to another helper keep the return unresolved.

A same-file function declaration may also have explicit early `if` returns
followed by a final top-level return. A final `switch` qualifies when every
case returns a fixed head and its default either throws directly or calls an
exact, stable same-file synchronous function whose body consists of one throw.
All returning paths must prove the same named or missing result. A caller may
append to the returned value only when every path returns a fresh builder from
the trusted SQL tag, including safe returned locals. A switch without a
default, a falling-through case, an unknown or mixed return, and async or
generator functions remain unresolved. A directly imported exported function
can carry this finite proof through the prepared one-hop projection.

One exact namespace destructure is also supported: a top-level `const`
unpacks a static property from a runtime `import * as services` binding, and
the local callable is used directly at configured executors. The prepared
resolver may follow explicit unique named reexports and a barrel's imported
default binding to a source-visible fixed-head helper. Its helper and builder
safety checks still apply. Computed keys, defaults, rest, mutable or escaped
namespace/local bindings, ambiguous exports, cycles, `export *`, missing
targets, and custom getter objects remain unresolved. Other reexport and
namespace forms do not gain this proof.

A trusted local builder returned by a helper also keeps its fixed head when a
captured closure may append suffixes before or after the return. The proof
requires every reference to that exact local binding to be a discarded static
one-argument append, a proven local append-only helper effect, or a direct
return from the binding's own function. Closure order and repeated calls do
not change the leading comment. Returning the builder from the captured
closure, using an append result, passing it to an unknown helper, aliasing or
writing it, or dynamic `eval`/`arguments` access leaves the return unresolved.
This proof applies only at the helper's own return; it does not make an escaped
builder safe at arbitrary executor reads.

One direct source import of an exported immutable string constant also qualifies when
its literal or template first segment has a complete fixed named head. A
complete missing head remains a finding. The prepared resolver follows the
exact import/export binding one hop; mutable exports, re-exports, unavailable
packages, ambiguous exports, and names formed by interpolation remain
unresolved. This value proof does not treat the constant as a callable helper.

Low can follow one exact source import when a helper calls an inline executor
callback with a proven SQL head. The helper directly exports a source-visible
function; its callback parameter may only be called with the same fresh
trusted `sql-template-strings` builder supplied by the caller, after static
`.append(oneExpression)` calls, or with a fixed named literal template. The
caller passes a direct inline `statement => read(statement)` or equivalent
configured executor callback. Every callback invocation must use the proven
payload. Unknown calls, aliases, writes, defaults, escapes, spreads, optional
calls, and source re-exports remain unresolved. Each caller is checked
separately.

A local `const` selecting between finite conditional arms qualifies when every
arm starts with a complete named literal comment. Tagged arms retain the
mutable-builder lifetime checks, including escapes before execution. An
unknown, partial, or missing arm remains unanalyzable; the syntax pass does
not evaluate the condition or combine SQL text.

Low also accepts a first comment name supplied by a bounded runtime string
set. A synchronous `for (const [name, other] of [["one", "x"], ...])` may
capture the exact read-only `name` binding in a callback when every direct
tuple row has a string name. A same-file helper may instead receive its first
parameter through one local `select` closure whose complete call set supplies
at most 16 direct string literals. The helper may append an ordinary template
with that name to an empty, exactly imported `sql-template-strings` builder;
its later safe appends do not change the first comment. Every substituted
prefix must close a nonempty `/* name */` before any other interpolation.
Aliases, writes, defaults, spreads, extra or dynamic calls, escaped helpers,
untrusted builders, and tagged bind interpolation remain unresolved. This
proof joins only the leading comment and does not compose the SQL suffix.

These are deliberately narrow low-effort syntax proofs. Low does not run the
helper/value evaluator, so other helper-produced heads can remain unresolved
when `effort: max` can recover them. A direct template tagged with an exact
default import (or its named `sql` import) from `sql-template-strings` keeps its fixed named head through
static `.append(...)` links with one opaque suffix expression. One local const
holding that chain also qualifies when its uses pass the builder safely to an
executor. Custom tags, optional or spread appends, and unknown leading heads
stay unresolved. The configured
`unanalyzable` policy applies to that low-effort uncertainty.

A local const initialized directly from that trusted named tag can also receive
later statement `.append(opaqueSuffix)` calls. The resulting builder qualifies
at later executor reads, including `query(statement.append(opaqueSuffix))`,
while its use remains in the same function and is neither captured nor passed
to another helper. References to the builder inside a suffix expression count
as an unsafe use.

A captured local builder also keeps its fixed named head when **every** runtime
reference to its exact binding is closed: a discarded static one-argument
`.append(...)`, a configured executor read, or a proven local append-only
effect. The closure may run before, after, or repeatedly; each permitted use
preserves the head. Exports, aliases, stores, returns, used append results,
property writes, other member reads, defaults that expose the builder, and
binding gaps remain unanalyzable. This proof applies only to low effort and
does not infer complete SQL or execute closures.

A discarded call to a same-file or direct imported function declaration may
also preserve that trusted local builder's prefix. The helper must receive the
builder as its first argument, and every use of that parameter must be a
discarded, static `.append(oneExpression)` chain. The helper may choose a
suffix at runtime. Defaults, aliases, captured parameters, used append results,
and other parameter uses leave the later read unanalyzable. Direct imported
helpers qualify only when their prepared source file exports the proven
function; unresolved and re-exported helpers remain unanalyzable.
One helper may call another proven same-file append-only helper with the same
first parameter as a complete expression statement. This composition is
bounded to four local edges and requires exact lexical call bindings; cycles,
used call results, aliases, and nested captures remain unresolved. A helper
may also `return query.append(suffix)` when that entire static append chain
is rooted at its first parameter and its caller discards the helper result.
The returned builder does not become a general value-flow proof.
Secondary parameters may use a direct object destructure with static keys and
plain bindings. Defaults, computed keys, and an argument expression that
captures or passes the caller's builder still leave the read unresolved.

For ordinary const tracing, its declaration and uses must stay in the same
function. The closed captured-root proof above is the narrow exception. Before the executor call,
its uses may only append supported literals or pass it directly to prepared
executors. An opaque suffix is supported for the trusted tagged initializer
or the trusted local builder described above. Other opaque appends make later
reads unanalyzable. Repeated
executor reads qualify. Reassignment, property writes, other receiver methods,
helper escapes, aliases, unsupported returns, stores, and exports make it unanalyzable
because a const binding does not make its builder immutable. An escape after
an executor call does not invalidate that earlier call.
A const holding a plain literal string retains its head when passed elsewhere;
passing an immutable string cannot change the original head.
Captured builders and loop calls that might observe a previous iteration's
escape stay unanalyzable.

A tagged-template left operand of `+` also keeps its literal head, directly or
through one local const. A tag on the right cannot repair an unknown left head.

The first complete block comment must contain a nonempty name. Parentheses and
TypeScript assertions do not add indirection. The const must resolve to the same
file's lexical declaration; a second alias, import, or destructured binding
does not qualify. Mutable bindings need the additional proof below.

Decidable literal heads without an annotation are violations at `low`, including
`query("SELECT 1")`, unnamed tagged templates, and one supported local const.
They remain failing findings under `report`, `advise`, and `ignore`. Complete
literal `BEGIN`, `COMMIT`, and `ROLLBACK` commands are exempt at both effort
levels. A transaction keyword at the end of a head followed by an unknown
suffix remains unresolved: the suffix could turn `BEGIN` into `BEGINNING`.

Other helper returns, conditional arms without proven heads, escaped builders, and
other unsupported shapes remain unanalyzable. Rewrite them as a leading named
literal at the executor call or as one local const initialized with that head
and passed directly.

The shared `unanalyzable` policy controls unresolved calls: `report` fails,
`advise` emits a non-failing advisory, and `ignore` skips them. `unanalyzableSql`
remains an alias. For example, set `options.effort: low` and
`options.unanalyzable: report` to require these forms. Shared configuration and
CLI overrides also apply.

Low-effort diagnostics use the executor call's line. Suppress at that call:

```ts
// no-mistakes-disable-next-line postgres-require-query-annotation
query(buildSql());
```

File and same-line directives also work. A helper-line suppression does not
suppress a low-effort call-site diagnostic. Other applications that remain at
`max` can still request evaluator work in the same invocation.

A stable synchronous helper may also return one fresh `{ text, values }` record
or one fresh trusted `sql-template-strings` builder. The helper must have a
single final return. A record needs an own, ordinary `text` property whose
string literal or template starts with a decidable head, plus an own `values`
property. The `text` property may instead read a same-function immutable local
string literal or template with a fixed leading head. Getters, spreads,
computed or duplicate fields, mutable text, and parameter-derived text remain
unresolved. A trusted builder return uses its exact imported tag.

One local `const` can hold that helper result for exactly the paired
`query(result.text, result.values)` read. Repeated calls, aliases, captures,
exports, writes, optional calls or fields, and additional uses remain unresolved.
A fresh plain record can also be extracted with exactly
`const { text, values } = helper(args)` and passed once as `query(text, values)`
in the same function and loop iteration. Renamed or extra fields, other uses,
and builder results remain unresolved in this form.
For a helper proven to return a fresh trusted builder, low effort also accepts
`const result = helper(args); query(result)` and
`query(helper(args).append(suffix))`. The local result may receive discarded
static one-argument appends before the executor; every other use, mutation,
capture, or export makes this projection unresolved. A plain `{ text, values }`
record or string result does not gain builder append behavior. The helper may
return one safe local trusted builder with a fixed head after append-only uses.
The same proof can follow one direct prepared source import; re-exports and
unavailable implementations remain unresolved. A definite missing annotation
still fails under every unanalyzable policy. This syntax proof does not request
the annotation evaluator or change max's append fallback. For unresolved results,
put a named literal at the executor instead of passing an opaque `.text` field.

For a configured relative factory or executor type, the helper head is used
only after the request resolver confirms that the executor import belongs to
the configured package. An unconfirmed relative call cannot make a local
builder's later use safe.

## Mutable literal bindings

At `low`, a decidable literal head is checked before the unanalyzable policy.
Unannotated string literals, no-substitution templates, tagged templates with a
known unannotated head, and one local const initialized with those forms are
findings even with `unanalyzable: advise` or `ignore`:

```ts
query("SELECT 1"); // Missing annotation: a failing finding, including with advise.
const statement = "SELECT 1";
query(statement); // The same definite violation.
// Fix: query("/* health/check */ SELECT 1");
```

A complete literal transaction command keeps the transaction exemption. An
incomplete head, such as ``query(`/* ${name} */ SELECT 1`)``, stays
unanalyzable. Helper returns and conditional heads outside the bounded forms above also remain unanalyzable. A same-function `let` or `var` used as the executor
argument can be decided when every direct assignment that may reach that call
is a literal string, no-substitution template, or tag without interpolations. Every such value
must have a known head: if all are named, the call passes; if any is missing
the annotation, the call is a finding. For example, an uninitialized or
undefined path contributes no SQL, while a mix of named and unannotated
literal assignments is still a finding:

```ts
function run(enabled: boolean) {
  let statement: string | undefined;
  if (enabled) statement = "/* health/check */ SELECT 1";
  read(statement); // The undefined path contributes no SQL; the literal path is named.

  let next = "/* health/check */ SELECT 1";
  if (enabled) next = "SELECT 2";
  read(next); // Definite missing annotation, including with unanalyzable: ignore.
}
```

An assignment from a helper call, a parameter, a captured/closure write, or a
mutation through an alias remains unanalyzable. Outside repeated control flow,
a future write does not affect an earlier executor call. A later write in the
same loop can reach the next iteration and is included. Initializers and
iterable expressions run once per loop entry; an outer
loop can still repeat that entry. The analysis conservatively considers direct
same-function assignments that may reach the call; it does not infer values
across calls or perform path-sensitive control-flow analysis.
Scalar literal bindings retain their heads through ordinary reads such as
logging. Tagged builders keep the direct-use restriction: a standalone
assignment can update the binding, but an assignment whose result is passed
elsewhere can expose the builder and remains unanalyzable. Rewrite
unresolved shapes as a named literal at the call or as one local `const`
initialized with that literal head and passed directly. Existing file and line
suppressions apply to definite findings too.
