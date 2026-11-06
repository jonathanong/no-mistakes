Saved foreign receiver work-budget controls.

Each `shared-N.mts` imports 64 distinct provider identities, declares N distinct
writer functions before the provider function, then invokes the providers and
writers. Each writer assigns `undefined` to `receiver.get`; `provide` assigns
every imported provider to the same mutable receiver. The ordinary module
constructs its sole Error subclass.

The source files are generated deterministically by listing provider0 through
provider63, write0 through write(N-1), and their calls in numeric order for
N = 16, 64, 2048, and 4096. Provider objects expose the built-in Error constructor.
No test writes fixture content at runtime.

The `argument-diamond-N.mts` cases forward a provider parameter through two
calls per level before storing it on the shared receiver. They exercise the
branching provider-expansion path with only a few kilobytes of saved source.

`scalar-wildcard.mts` projects a wildcard path over a shared object of scalar
values. It exercises node-visit accounting even when projection emits no
references. The saved forward-receiver controls in
`../../value-flow-forward-receiver-alias/fixture/src/budget-N.mts` contain 128
or 256 distinct writers over a 20-member deferred receiver path; both stop at
a reduced test budget and preserve complete facts without that limit.
