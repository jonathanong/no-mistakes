import { read, sql } from '@example/db';
function runLeft() {
  const query = sql`/* returns/left */ SELECT 1`;
  query.append(['; DELETE FROM records'].join(''));
  read(query);
}
function runRight() {
  const query = sql`/* returns/right */ SELECT 2`;
  query.append(['; DELETE FROM records'].join(''));
  read(query);
}
// The lexical walker cannot reconstruct the joined append; precise entrypoint
// evaluation must execute either private function returned by this public factory.
export function get(flag: boolean, nested: boolean) {
  if (flag) {
    if (nested) return runLeft;
    return runRight;
  } else {
    return runRight;
  }
}
