import { read, sql } from '@example/db';

// Rest parameters intentionally keep this helper unsupported. Its append must
// prevent a legacy return prefix from reviving a potentially modified builder.
function unsupportedAppend(query: unknown, ...unused: unknown[]) {
  query.append(sql` DELETE FROM records`);
  return query;
}
export function unsupportedAppendIsNotAReadProof() {
  const query = sql`SELECT 1`;
  read(unsupportedAppend(query));
}
