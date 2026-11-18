import { readRows, transaction } from './db.mjs';
import { page } from './pager.mjs';
import { sql, type SqlStatement } from './sql.mjs';
export const inlineInTransaction = () => transaction(async tx => {
  const { rows } = await tx(sql`/* inlineInTransaction */ UPDATE widgets SET seen = TRUE RETURNING id`);
  return rows;
});
export async function topLevel(query: string) {
  const { rows } = await readRows(sql`/* topLevel */ SELECT id FROM widgets`, { query });
  return rows;
}
function widgetsStatement(): SqlStatement {
  return sql`/* viaBuilder */ SELECT id FROM widgets`;
}
export async function viaBuilder() {
  const { rows } = await readRows(widgetsStatement());
  return rows;
}
const constStatement = sql`/* viaConst */ SELECT id FROM widgets`;
export async function viaConst(query: string) {
  const statement = constStatement;
  const { rows } = await readRows(statement, { query });
  return rows;
}
export const pagedBeside = (after?: string) => page(sql`/* pagedBeside */ SELECT id FROM widgets WHERE TRUE`, after, 50, statement => readRows(statement));
