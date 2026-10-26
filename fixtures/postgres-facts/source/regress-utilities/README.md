# PostgreSQL 17 utility witnesses

`valid.sql` retains every utility witness in #2754, in issue order, from
`postgres/postgres` revision `bd807012669ed8573317562b78c1221e4fabe373`.
The test reads each statement separately: COPY FROM STDIN is a command grammar
witness here, not a multi-line data stream.

`variants.sql` covers complete optional clauses and lists. `malformed.sql`
contains PostgreSQL-parser-rejected near misses which the baseline analyzer also
reports. The validator must not turn those existing failures into advisories.
`rule-actions.sql` protects action-list semicolons and recovery of a later table.
