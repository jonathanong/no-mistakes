import { read } from './db.mjs';
import { page, replaced, prepended, nested, escaped, pageRows, pageBound, pageStatement, otherBuilder, pageDefault, pageComputed, pageAfter, pageMutate, plainDestructure } from './pager.mjs';
import { sql, type SqlStatement } from './sql.mjs';
export const first = (after?: string) => page(sql`/* pageWidgets */ SELECT id FROM widgets WHERE TRUE`, after, 50, statement => read(statement));
export const second = (after?: string) => page(sql`/* pageGadgets */ SELECT id FROM gadgets WHERE TRUE`, after, 50, statement => read(statement));
export const missing = () => page(sql`SELECT id FROM widgets`, undefined, 50, statement => read(statement));
export const opaque = (value: SqlStatement) => page(value, undefined, 50, statement => read(statement));
export const replacement = () => replaced(sql`/* replacement */ SELECT id FROM widgets`, statement => read(statement));
export const prepend = () => prepended(sql`/* prepend */ SELECT id FROM widgets`, statement => read(statement));
export const early = () => {
  statement.append(sql` LIMIT 10`);
  const statement = sql`/* early */ SELECT id FROM widgets`;
  return page(statement, undefined, 50, statement => read(statement));
};
async function localPage<T>(selectWhere: SqlStatement, afterId: string | undefined, limit: number, query: (s: SqlStatement) => Promise<T>) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  return await query(selectWhere);
}
export const local = (after?: string) => localPage(sql`/* local */ SELECT id FROM widgets`, after, 50, statement => read(statement));
export const shared = () => {
  const statement = sql`/* shared */ SELECT id FROM widgets`;
  return page(statement, undefined, 50, value => read(value));
};
export const nestedTails = (after?: string) => nested(sql`/* nested */ SELECT id FROM widgets`, after, 1, statement => read(statement));
export const escapedInput = (after?: string) => escaped(sql`/* escaped */ SELECT id FROM widgets`, after, statement => read(statement));
export const destructured = (after?: string) => pageRows(sql`/* destructured */ SELECT id FROM widgets`, after, 50, statement => read(statement).then(() => ({ rows: [] })));
export const bound = (after?: string) => pageBound(sql`/* bound */ SELECT id FROM widgets`, after, statement => read(statement));
export const bare = (after?: string) => pageStatement(sql`/* bare */ SELECT id FROM widgets`, after, statement => read(statement));
export const differentBuilder = (after?: string) => otherBuilder(sql`/* appended */ SELECT id FROM widgets`, sql`SELECT id FROM gadgets`, after, statement => read(statement));
export const defaultMutates = (after?: string) => pageDefault(sql`/* defaultMutates */ SELECT id FROM widgets`, after, statement => read(statement).then(() => ({ rows: [] })));
export const computedKey = (after?: string) => pageComputed(sql`/* computedKey */ SELECT id FROM widgets`, after, statement => read(statement).then(() => ({ rows: [] })));
export const appendAfter = (after?: string) => pageAfter(sql`/* appendAfter */ SELECT id FROM widgets`, after, statement => read(statement).then(() => ({ rows: [] })));
export const mutateAfter = (after?: string) => pageMutate(sql`/* mutateAfter */ SELECT id FROM widgets`, after, statement => read(statement).then(() => ({ rows: [] })));
export const plainRows = () => plainDestructure(sql`/* plainRows */ SELECT id FROM widgets`, statement => read(statement).then(() => ({ rows: [] })));
