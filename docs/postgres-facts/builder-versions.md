<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

# Builder versions and conservative recovery

If any possible outcome is opaque, if there are more than 16 versions, or if
recovery exceeds the depth limit of 8, no partial version set is published.
Loops other than the bounded helper loops described [below](imported-and-literal-identifier-helpers.md#helper-control-flow-and-nested-composition),
untagged `${...}` SQL-text interpolation other than the
[literal-identifier helpers](imported-and-literal-identifier-helpers.md#imported-and-literal-identifier-helpers)
and a ternary, `&&`, `||`, or `??` whose every reached branch is a literal or a
same-file immutable `const` of those literals (nested ternaries and logical
expressions included),
calls to imported helpers the request resolver cannot prove (see the same
section for what it does resolve), `sql.raw(variable)`, and `sql.join` over
non-literal input, spreads, or array holes remain `Dynamic` and retain the
existing `unanalyzableSql` behavior.
That clause expression is one alternative per reached arm, under the same
16-version and depth-8 caps. An unknown reached arm (a parameter, call, `let`,
reassignment, or anything else) or a cap overflow publishes no alternative.
`&&` and `||` use string truthiness: only `""` is a falsy clause. A branch
JavaScript will not evaluate is not part of the union, so `"AND id > $2" ||
unknown` and `"" && unknown` stay the known string. A reached unknown arm,
including `unknown || "AND id > $2"` and `unknown && "AND id > $2"`, publishes
nothing. `??` keeps a proven string, including `""`, and does not take its
fallback; an unknown left still publishes nothing. A non-string result is not
stringified into the clause.
An unknown falsy result from `condition && sqlFragment` cannot prove whether
a following `??` takes its fallback, so that composed call remains `Dynamic`.
Present and absent fragments inside trusted templates and recoverable `||`
fallbacks remain supported.
Stored parameter-helper returns also remain opaque: they alias a mutable builder
that can change after the helper call, so its recovered text is not a snapshot.
After a known mutating helper call, later reads of its caller binding and tracked
aliases remain `Dynamic`, retaining only verified leading SQL. When branch
assignments lose the exact builder identity, the extractor conservatively
invalidates the connected group of simple identifier aliases, including
historical aliases.
Passing a tracked mutable builder to an ordinary call without a verified summary
invalidates complete SQL across tracked aliases, including the legacy
`kind` and `sql_text` fields. Calls evaluated inside `if` tests, conditional
tests, or logical left operands also clear the affected SQL prefix and keep later
reads `Dynamic`. Literal array and object arguments retain these alias effects;
merely declaring a callback does not execute its body. Recognized executors,
trusted constructors, and known builder appends retain their existing behavior;
copied SQL strings and unrelated builders remain independent.
Relative factory and type candidates are not confirmed consumers during
extraction when `importSpecifier` is configured. Their pending executor calls
capture SQL before invocation, and prepared resolver projection later keeps
only owners inside the configured package. Passing a mutable builder to such a
candidate clears finite alternatives for subsequent reuse and tracked aliases,
because a rejected owner can mutate that builder. This is deliberately
conservative even when projection later accepts the owner: it does not replay
the extractor to restore discarded alternatives. The pending call's captured
SQL, including proven mutations in later arguments, and source order stay
intact: the call captures its arguments before applying its own consumer
effects. Copied SQL strings, unrelated builders, and
already-confirmed direct executors retain their facts; ordinary-call
invalidation also discards complete legacy `kind` and `sql_text` snapshots. Subsequent
branching builder calls without complete alternatives follow each rule's
existing `unanalyzableSql` policy.
Finite recovery treats mutable SQL bindings and builders captured from an
enclosing scope as opaque: the extractor cannot prove their state when the
nested function executes. Branching calls that depend on those captures remain
`Dynamic`. Recoverable `const` primitive SQL strings, including conditional
string alternatives, retain finite recovery. This captured-state fallback alone preserves the legacy
`kind` and `sql_text` fields of non-branching calls. An escaped captured builder
separately follows the [mutable-builder boundary](../postgres-builder-mutations.md).
Ordinary `.append()` mutations clear finite-version snapshots cached by other
tracked aliases to the same mutable builder. Branching calls that depend on a
cleared snapshot stay `Dynamic`; the directly updated binding can retain
recoverable alternatives. Other aliases lose complete legacy `kind` and
`sql_text` snapshots too, including non-branching calls. Replacing a
binding with a fresh builder keeps that new builder independent. These alias
rules do not apply to copied strings changed with `+=`.
Conditional tests, logical left operands, and earlier template interpolations
that mutate an existing SQL binding also stay `Dynamic` when recovery would
otherwise consume its earlier snapshot later in the same expression.
Trusted template tags run after every interpolation has been evaluated. A final
interpolation that mutates a captured builder also makes the call `Dynamic` when
an earlier interpolation captured its mutable fragment reference. Exact builder
identities distinguish independent objects, including fresh replacements that
retain historical alias links. When an identity is missing, connected tracked
aliases keep the check conservative. Mutations to a separate builder or a fresh
helper argument retain finite recovery. Aliases created within the final
expression are followed before later effects. Unsupported assignment targets
and value wrappers whose identity writes cannot be proved stay conservative.
Calls also stay conservative when argument, property, or element evaluation
can change a later builder lookup or a retained helper result. Earlier arguments
and receivers keep the objects captured before subsequent writes. Direct appends
and known or opaque helper effects use this boundary. Primitive bind values are
already copied, so reassigning their bindings in the final expression does not
invalidate them. Ordinary final binds, a single mutating fragment interpolation,
and appends to fresh nested fragments retain their existing recovery.
Pure identifier reassignment in the final interpolation also preserves an
earlier captured fragment: replacing the binding does not change that object.
Later reads can recover the replacement while its snapshot remains available;
existing conservative alias invalidation still applies. Mutations or escapes on
the assignment's right-hand side still make the call `Dynamic`.
An append or mutating helper that consumes the builder returned by an assignment
also stays `Dynamic`; assigning the reference does not copy the builder.
Executor arguments follow evaluation order. Proven later mutations of the same
directly passed builder use its final state, including conditional or logical
appends; primitive SQL arguments keep their earlier value copies. Uncertain
builder selection or identity, opaque helper or invoked-closure effects, and
conditional replacement of the original binding stay `Dynamic`. Passing a
callback without invoking it does not execute its body.
Switch discriminants or case labels that mutate an existing SQL binding also
remain opaque because earlier labels can change the state of later cases.
Any assignment or update in a case label, including destructuring or a directly
invoked arrow/function body, also disables finite recovery inside that switch.
Literal closures invoked through `.call(...)` or `.apply(...)`, including
literal computed properties and TypeScript wrappers, follow the same boundary.
The detector inspects callee and argument writes before the invoked body;
callback arguments and closures merely referenced or bound are not invoked.
Failed labels can change scalar guards before a later case runs; a default in
the middle runs only after later labels fail. This fallback scans each label and
body once without enumerating or replaying writes along fallthrough paths.
Each case starts from conservative entry state, so a failed case body cannot
supply complete SQL to a later direct-entry case. Bindings changed by an earlier
body lose their recovered prefix as well as their finite versions: direct entry
and fallthrough can see different statements, including different DML kinds.
Merely creating or passing a callback does not execute its body. Literal SQL
arguments and unchanged legacy constants retain their existing checks, while
branching calls use the rule's `unanalyzableSql` policy.
Invoking a generator in a label creates an iterator without executing its body.
Its evaluated arguments and parameter defaults still count as effects. A visibly
unused iterator therefore does not discard finite SQL versions merely because
the generator body contains a write. Iterator consumption through a member call
(including `.next()`), `for...of`, array or argument spread, tagged template
interpolations, or `yield*` retains the conservative body check, including
iterators returned by invoked literal
closures. Iterators stored in bindings, including parameter and destructuring
defaults, and unknown consumers keep that conservative check without tracing
aliases or iterator steps; even a stored unused iterator may remain opaque.
Ordinary invoked functions and arrows retain their body checks.
Plain template string coercion does not advance a fresh native iterator.
`try`, `catch`, and `finally` phases are inspected once, but exception paths are
not enumerated. Finite execution snapshots are discarded before handlers and
after each phase because an earlier operation may have thrown before completing.
SQL bindings changed during a phase also become `Dynamic`, retaining only a
verified prefix where available; later queries cannot treat their post-mutation
text as complete SQL. Untouched legacy fields stay unchanged. Statements
initialized or assigned afresh in a handler and literal executor arguments
remain recoverable within that handler.

Each `EmbeddedSqlVariant` records `sql_text`, `line`,
`sql_source_positions`, and `recovered_placeholder_positions`. Its source map
also records `sql_source_offsets`, one physical byte origin per recovered SQL
byte, including decoded escapes and renumbered generated placeholders. This
identity distinguishes separate clauses on the same source line and deduplicates
a shared clause even when an optional earlier fragment changes its SQL column.
The mapping
retains the physical lines and columns of SQL inside branch fragments, so
findings and `no-mistakes-disable-*` directives apply there. A violation in
any version is reported; repeated findings at the same physical location for
the same rule and target are deduplicated and output stays deterministic.
An executor-line suppression applies only to that invocation. A suppressed
occurrence does not consume a shared finding's identity: another unsuppressed
execution of the same SQL still reports the violation once. A directive on a
shared SQL clause suppresses that physical clause wherever it is reused.
Shape policies also inspect unexecuted builder fragments. An executor
suppression covers the exact append occurrences that contributed to that
execution, including appends in nested fragments and branch-assigned builders.
Another append occurrence that reuses the same SQL token remains checked.
Lock-ordering checks each version independently, so ordering on one path
cannot make another path safe.

The legacy `kind` and `sql_text` fields keep their earlier meaning. An
enumerated call can still have `kind: Dynamic`; consumers must prefer its
nonempty `variants` to the legacy partial text. Ordinary calls that already
had fully static SQL keep an empty version list and their existing facts.

Rust callers constructing `EmbeddedSqlCall` with a struct literal must add
`variants: Vec::new()` for an ordinary call, or use
`..EmbeddedSqlCall::default()` for unspecified fields. Derived equality and
`Debug` now include the version list. The legacy `kind` and `sql_text` fields
retain their contract; Node source-fact and check JSON shapes remain unchanged.

[PostgreSQL fact sources](../postgres-facts.md)
