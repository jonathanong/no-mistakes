// The callback worklist retains the removed child while its earlier sibling branches.
import { read, sql } from '@example/db';
declare function opaque(...values: unknown[]): void;
function change(values, flag: boolean) {
  values[1] = 0;
  if (flag) return ['left'];
  return ['right'];
}
export function worklist(flag: boolean) {
  const values = [() => change(values, flag), [() => {
    const query = sql`/* transient/replaced-child */ SELECT 9`;
    query.append('; DELETE FROM records');
    read(query);
  }]];
  opaque(values);
}
