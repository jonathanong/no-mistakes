# Sparse branch writes

This saved fixture retains 64 distinct annotated SQL strings while 32 unknown
ternary alternatives each write a fresh mutable binding. Each left arm is an
`OpaqueWrite` expression rather than an unsupported assignment statement.
The ternaries remain const initializers: stand-alone effect statements
conservatively invalidate every prefix and would erase this proof.
The test asserts 32 collected alternatives and no unsupported/statement-branch
steps before evaluating either mode. `retained` must remain the specific
`/* stable00 */ SELECT 0` value in both legacy and precise-record modes. The
second query deliberately lacks an annotation, so a real check reports exactly
one finding; replacing all values with Unknown does not satisfy the control.

The binding-join unit test compares all semantic evaluator state with the
frozen legacy join and counts actual binding comparisons. The implementation
follows dirty-key journals only when both snapshots have a proven path to the
original. Generic map writes, unrelated snapshots, and bounded-history eviction
retain the original borrowed full-scan fallback.

A journal starts only on a keyed copy-on-write mutation of a scope with at
least 32 bindings. Smaller scopes retain the borrowed scan: journal allocation
cost outweighs the avoided comparisons on tiny branches. Cold scopes without a
journal allocate none. Existing shared journals detach on a later write to
preserve their snapshots. A journal retains weak snapshot identities and at most
33 history nodes / 256 key names, without keeping historical value maps alive.
The optional journal lives beside the shared bindings pointer, preserving the
existing binding-allocation layout and adding one pointer to each scope handle.
Insert, delete and mutable lookup record names; bulk mutation invalidates the
journal. Weak origins prevent a subsequently mutated snapshot from masquerading
as an earlier state at the same address.

Work-count assertions require reduced comparisons on this input. Release-mode matched
wall/CPU/RSS measurements and full-consumer findings parity are separate gates;
this fixture establishes no hosted full-check budget pass.
