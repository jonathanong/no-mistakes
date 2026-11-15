import sql from 'sql-template-strings';
import { write } from 'db';

declare const ids: string[];

function keysSql(o: { boost: boolean }) {
  return sql`WITH k AS (SELECT id FROM t WHERE flag = ${o.boost}) `;
}

export async function run() {
  await write(
    sql`/* q */`.append(keysSql({ boost: true })).append(
      sql` SELECT id FROM k WHERE id = ANY(${ids}::uuid[]) ORDER BY id LIMIT 3`,
    ),
  );
}
