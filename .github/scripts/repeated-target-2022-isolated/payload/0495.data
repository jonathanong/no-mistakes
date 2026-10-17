import { read, sql } from '@example/db';
function runOpaque() {
  const query = sql`/* returns/opaque */ SELECT 6`;
  query.append(['; DELETE FROM records'].join(''));
  read(query);
}
function runUnsupported(input: unknown[]) {
  const query = sql`/* returns/unsupported */ SELECT 7`;
  for (const value of input) query.append('; DELETE FROM records');
  read(query);
}
// An opaque factory call is not a proven reference to its callback argument.
export function opaque(flag: boolean) {
  if (flag) return unknownFactory(runOpaque);
  else return null;
}
// Escaping a function does not make its unmodeled loop safe to execute precisely.
export function unsupported(flag: boolean) {
  if (flag) return runUnsupported;
  else return null;
}
