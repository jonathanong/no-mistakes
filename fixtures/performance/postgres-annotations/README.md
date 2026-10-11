# PostgreSQL annotation benchmark

Twelve route modules share two helpers across fifteen source files. The scalar
helper has four independent conditional declarations. The heap helper has two
callbacks, one selected callback, and a cyclic `arguments` object. Each route
calls the two helpers once and passes their mixed scalar/reference results to
an opaque consumer before checking one annotated query and one unannotated
query. The expected report has twelve findings and no warnings.

Keep the scalar arrays and argument cycle: they distinguish immutable scalar
work that reachability can skip from live callback frames and heap identities
that branch compaction must retain. Keep scalar branches independent so nested
joins do not dominate this small benchmark. Source files are saved here;
benchmark runs never generate input files.
