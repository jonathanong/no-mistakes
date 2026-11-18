import bindSql from 'sql-template-strings';
import { read } from './db.mjs';
import { sql, type SqlStatement } from './sql.mjs';
import { bound, mutated, shadowed, page } from './pager.mjs';
import { sql as otherSql } from './other-sql.mjs';
import { foreignStatement } from './other/source.mjs';

export const boundFragment = () => bound(bindSql`/* bound */ SELECT id FROM widgets`, statement => read(statement));
export const mutation = () => mutated(sql`/* mutation */ SELECT id FROM widgets`, statement => read(statement));
export const shadow = (tag: typeof sql) => shadowed(sql`/* shadow */ SELECT id FROM widgets`, tag, statement => read(statement));
export const computed = (specifier: string) => import(specifier).then(({ page }) =>
  page(sql`/* computed */ SELECT id FROM widgets`, undefined, (statement: SqlStatement) => read(statement)),
);

export const foreignBuiltin = () => page(bindSql`/* foreignBuiltin */ SELECT id FROM widgets` as unknown as SqlStatement, undefined, statement => read(statement));
export const foreignCustom = () => page(otherSql`/* foreignCustom */ SELECT id FROM widgets`, undefined, statement => read(statement));

export const relativeForeign = () => page(foreignStatement(), undefined, statement => read(statement));
