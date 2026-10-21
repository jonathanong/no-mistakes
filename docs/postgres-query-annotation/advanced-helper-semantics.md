# Advanced query annotation helper semantics

Speculative function entrypoints use isolated initialized module state. Actual helper and callback call chains retain shared builder state. Bare `var` redeclarations preserve existing parameters and hoisted functions. Logical and conditional expressions contribute every syntactically possible helper invocation; helper calls in unsupported syntax remain conservative rather than allowing a favorable modeled call to hide unknown arguments. Diamond star re-exports of the same original binding resolve to that binding.

Opaque mutations invalidate previously proven prefixes of mutable builders, including method receivers, spread arguments, nested containers, property writes/deletes, and constructor inputs. Replacing or deleting `String.raw` revokes built-in tag trust; writes to a lexically shadowed `String` leave the global built-in unaffected. Spread arguments remain conservative for positional helper and callback substitution.

Conditional and logical arms use independent mutable state. An unchanged leading
annotation remains provable, while a possible opaque mutation makes the SQL
unanalyzable. Unsupported control flow containing potentially mutating calls
cannot restore an earlier prefix through legacy recovery. A hoisted local `var`
shadows a captured binding, including when it has no initializer.

An initialized `var` redeclaration assigns when its initializer executes, preserving
a parameter value used earlier. Regular helpers own an `arguments` object; arrows
inherit it from their enclosing helper, so opaque mutations through that object
retain the same builder identity. Later arguments can update builder aliases
already evaluated for the same call. Nested callbacks retain live inherited
bindings, so a later initialized `var` assignment updates their captured value.
Binding updates use an index of their actual readers rather than scanning unrelated
invocation frames.
Unreachable invocation objects and frames are removed before speculative entrypoint
snapshots, including calls interleaved with imported helper initialization.
Static numeric and canonical numeric-string
indices select the corresponding argument; dynamic indices remain conservative.
In sloppy functions with simple parameters, assigning a parameter updates its
mapped argument slot; opaque slot mutations make mapped parameters unknown. Reading
an immutable parameter through opaque syntax preserves its value; unsupported
binding writes remain conservative, including rebinding `arguments`. Replacing a
known argument slot preserves the builder formerly referenced by that slot and
other indexed arguments; prior opaque escapes remain uncertain. An unescaped
invocation object's `length` is a primitive and does not expose its slot values.
Static writes beyond the original argument count use sparse slots rather than
resizing the invocation object; storing a reference does not mutate its builder.
Unsupported assignments evaluate their RHS before invalidating written bindings.
Destructuring targets are bindings, while computed keys, defaults, and member
targets retain their evaluation effects. Conditional callbacks retain imported
module state initialized in their possible arms, while sibling arms start from
the same pristine state. Rebinding one variable leaves
other aliases to its former builder intact. Opaque callback traversal also follows
possible returned callbacks and newly installed argument-slot callbacks within
the existing bound. Assigning a callback into a live or recreated slot does not invoke it unless
the argument object already escaped. Reentrant callback traversal shares an
invocation set within one opaque consumer; separate consumers remain independent.
Conditional branches start from the same invocation state, so analyzing one branch
does not suppress a callback in another branch. A callback is considered consumed
after the join only if every possible branch invoked it.
A stateful callback may be revisited when its observed captured inputs change,
within the same depth bound and with a guard against reentrant execution.
Observed inputs include imported helpers' captured module state; shared read
projections are reused within one callback traversal.
Writes through a still-mapped formal also expose a newly installed callback when
its argument object previously escaped. A known unannotated scalar slot value
remains a violation alongside an unknown mapping possibility, even in ignore mode;
an annotated possibility does not prove that every possible value is safe.
Differing dense or sparse values of one argument slot across branches stay
possible candidates, so an unannotated arm remains a violation when
`unanalyzableSql` is `ignore`.
Deletion preserves known slot possibilities before disconnecting a formal. A
dynamic deletion may disconnect a formal, so later slot writes retain both its
previous value and the assigned value as possibilities.
Static indexed assignments also follow argument-object aliases; assignments to
other receiver values remain conservative.
Strict functions keep separate bindings. A later definite parameter assignment establishes
its new value until another escape of the argument object makes it uncertain.
Deleting a known slot disconnects its mapping
only when every possible branch deletes it. A later property write makes that
index readable again without reconnecting the formal parameter, including when
the deletion itself happened on only one branch. The recreated property is the
assigned value; a dynamic deletion of an unwritten index stays uncertain. Deletion after an opaque escape
preserves earlier uncertainty instead of restoring the original parameter value.
Spread calls have unknown argument positions. Passing the complete argument object
to opaque code makes its slots unknown, while an already copied immutable value
retains its value. Escaped slots still retain possible callback effects, without
proving their SQL value. Argument aliases share one invocation object; reading its
primitive `length` does not expose its slots. Appending a builder refreshes its aliases inside argument
objects and promises. Deleting an argument slot makes the slot unknown while
preserving other slots and the builder that the slot referenced. Deleting a known
non-index property leaves indexed slots intact; dynamic keys remain conservative. Computed-key
effects still run, including inside a `void` wrapper. Comma expressions expose only
their last value. Pure unary numeric indices such as `+0` and `-0` address slot zero.
Non-coercive `void`, `typeof`, and `!` expressions discard
callback values while evaluating operand effects. An ordinary property read
inside one of those wrappers can still run a getter, so captured builders do
not stay proven. Reads of a proven argument object's numeric slots and
`length`, including an alias, stay data reads. A named property of that object,
such as `arguments.trigger`, can be an accessor. A further property on one of
those slots can still run a getter. Numeric `+`, `-`, and `~`
coercion remains conservative because conversion hooks can mutate operands. A non-arrow function called through a member or index receives that base as
`this`. An arrow keeps its lexical `this` and ignores the call receiver. A bare
call does not reuse a captured `this`. Named function self-bindings are shadowed by
invocation parameters and the ordinary `arguments` binding.
Untrusted local tags can mutate captured
builders even when their own bodies use supported straight-line syntax. They can
also invoke interpolated callbacks, including callbacks from conditional or logical
arms, invalidating captured builder prefixes.
