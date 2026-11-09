# Request-owned fact narrowing demand

Integration discovery demand belongs to one logical check request. Its prepared
analysis session owns the selected discovery paths; the prepared collector borrows
that descriptor explicitly. Helpers retain their global demand. Ordinary public
collectors always request full discovery, including when their session also owns
a prepared descriptor. No ambient root-to-scope lookup changes another request.

Batch requests carry their discovery descriptor once. Each existing request index
selects its descriptor when creating a file projection; per-file fact plans do
not carry another path set. A scoped and an unrestricted request for the same
source retain separate projections over one parsed program, in either request
order. Runner helper and cached configuration projections carry that same
explicit demand. Parse-error recovery retains its original diagnostics.

The existing `FamilyPathScope::Only(Arc<BTreeSet<PathBuf>>)` constructor and public
collector signatures remain unchanged. Disabled integration skips descriptor
lookup and allocation. Neither discovery demand nor SQL audit policy adds a field
to the common `CheckFactPlan`.

SQL suppression-audit demand belongs to each declared SQL scan. An active audit
marks every consuming scan and any marked scan prevents SQL narrowing. Matching
scan applications coalesce and retain the audit requirement by logical OR. A
consumer with no declared SQL scans contributes no SQL suppression inventory.

This prototype reorganizes request ownership. It does not establish the cause of
small control timing differences or claim a performance improvement. Paired
profiling and the full consuming/control measurement gate remain necessary.
