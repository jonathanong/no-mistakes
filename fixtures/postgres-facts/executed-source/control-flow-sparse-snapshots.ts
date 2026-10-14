import { read, sql } from '@example/db';
function sparse() {
  const parts = [];
  parts[9] = 'x';
  if (parts.length === 0) return '/* snapshots/sparse */ SELECT 1';
  return '/* snapshots/sparse */ WITH changed AS (DELETE FROM records RETURNING id) SELECT id FROM changed';
}
export function sparseBranch() { read(sparse()); }
function appendWrite(query: unknown) { query.append('; DELETE FROM records'); return 0; }
function first(parts: unknown[]) { return parts[0]; }
export function liveArrayBuilder() {
  const query = sql`/* snapshots/live */ SELECT 1`;
  const parts = [query, appendWrite(query)];
  read(first(parts));
}
