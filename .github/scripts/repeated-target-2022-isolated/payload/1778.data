import { read, sql } from '@example/db';
const query = sql`/* hoisting/captured */ SELECT 1`;
function shadowedByVar() {
  // A var in an unreachable arm still shadows the captured builder.
  if (false) { var query; }
  read(query);
}
function unaffectedByLet() {
  // A block-scoped declaration in an unreachable arm does not shadow this use.
  if (false) { let query; }
  read(query);
}
export function compareHoisting() {
  shadowedByVar();
  unaffectedByLet();
}
