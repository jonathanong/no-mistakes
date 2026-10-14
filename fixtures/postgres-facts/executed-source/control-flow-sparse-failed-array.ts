import { read, sql } from '@example/db';
export const query = sql`SELECT 1`;
function appendKnown(target: unknown) { target.append(sql`; DELETE FROM records`); }
export function sparseBranch(flag: boolean, input: unknown[]) {
  const callback = () => { for (const value of input) appendKnown(query); read(query); };
  const parts = [];
  // The largest valid array index stays sparse, even when the scope loses proof.
  parts[4294967294] = callback;
  if (flag) {
    const trigger = 0;
    parts[4294967294]();
  }
}
