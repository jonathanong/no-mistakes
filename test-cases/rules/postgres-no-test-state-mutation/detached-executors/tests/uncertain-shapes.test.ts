import { pool } from '@example/db';
// A default or unknown property retains provenance without certifying the callable.
const { query: fallback = replacement } = pool;
fallback('SELECT 1');
const { [runtimeKey]: uncertain } = pool;
uncertain('SELECT 1');
const { other: unrelated } = pool;
const { query: { nested } } = pool;
