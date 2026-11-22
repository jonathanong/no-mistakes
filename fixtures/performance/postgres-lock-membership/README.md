# PostgreSQL lock membership benchmark fixture

Synthetic TypeScript input for comparing transaction-membership advisories in
`postgres-lock-written-rows-only`. It contains four copies of each of the first
four cases and one configured `beginTransaction` control:

- same-statement lock and write on `child`;
- transaction-only executor parameter with a later matching write;
- transaction-only executor parameter with no write;
- pool-or-transaction executor union with a later write;
- configured `beginTransaction` control with a later matching write.

Measure baseline and candidate with distinct binaries built from their
respective revisions, using the same fixture, config, flags, and environment.
Count advisories specifically for transaction membership and also report the
total unresolved advisory count.

Measured against main `eec414877b4c1c22aae47deab974cab3432390cf` and this change,
with one and two analysis jobs: transaction-membership abstentions fall from
16 to 8. Both reports have zero failing findings and zero warnings. The eight
remaining cases are unmatched caller-owned locks and pool-or-transaction unions.
