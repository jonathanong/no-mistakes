import { sql, type SqlStatement } from './sql.mjs';
import { read } from './db.mjs';
import { page } from './pager.mjs';
type Run = (s: SqlStatement) => Promise<void>;
function shadowed(statement: SqlStatement, sql: typeof import('./sql.mjs').sql, after: boolean, run: Run) {
  if (after) statement.append(sql` LIMIT 10`);
  return run(statement);
}
export const shadow = () => shadowed(sql`/* shadow */ SELECT id FROM widgets`, sql, true, s => read(s));
declare function opaque(): boolean;
function condition(statement: SqlStatement, run: Run) {
  if (opaque()) statement.append(sql` LIMIT 10`);
  return run(statement);
}
export const unsafeCondition = () => condition(sql`/* condition */ SELECT id FROM widgets`, s => read(s));
function callbackInBranch(statement: SqlStatement, after: boolean, run: Run) {
  if (after) run(statement);
  return run(statement);
}
export const branchCallback = () => callbackInBranch(sql`/* branch */ SELECT id FROM widgets`, true, s => read(s));
const run = (statement: SqlStatement) => read(statement);
export const knownSharedCallback = () => page(sql`/* known */ SELECT id FROM widgets`, undefined, 50, run);
export const opaqueSharedCallback = (statement: SqlStatement) => page(statement, undefined, 50, run);
function twice(statement: SqlStatement, after: boolean, run: Run) {
  if (after) statement.append(sql` LIMIT 10`);
  run(statement);
  return run(statement);
}
export const repeated = () => twice(sql`/* twice */ SELECT id FROM widgets`, true, statement => read(statement));
declare function replaceHead(statement: SqlStatement): boolean;
function defaultParameter(statement: SqlStatement, run: Run, after = replaceHead(statement)) {
  if (after) statement.append(sql` LIMIT 10`);
  return run(statement);
}
export const defaultEffect = () => defaultParameter(sql`/* default */ SELECT id FROM widgets`, statement => read(statement));
function rawShadow(statement: SqlStatement, String: { raw: Function }, run: Run) {
  if (true) statement.append(String.raw` LIMIT 10`);
  return run(statement);
}
export const rawParameter = () => rawShadow(sql`/* raw */ SELECT id FROM widgets`, String, statement => read(statement));
export const namedFunction = () => {
  const helper = function sql(statement: SqlStatement, run: Run) {
    if (true) statement.append(sql` LIMIT 10`);
    return run(statement);
  };
  return helper(sql`/* named */ SELECT id FROM widgets`, statement => read(statement));
};
