import { query } from '@db';
// Constants keep SQL locations distinct from the call-site directives.
const unknownOwner = 'SELECT 1 FROM missing WHERE id::text = $1';
const ambiguousOwner = 'SELECT 1 FROM users, sessions WHERE id::text = $1';
// no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates
query(unknownOwner, []);
query(ambiguousOwner, []); // no-mistakes-disable-line postgres-no-transformed-uuid-predicates
// Separate constant locations prevent finding deduplication from hiding a suppression failure.
const unknownControl = 'SELECT 1 FROM missing WHERE id::text = $1';
const ambiguousControl = 'SELECT 1 FROM users, sessions WHERE id::text = $1';
query(unknownControl, []);
query(ambiguousControl, []);
