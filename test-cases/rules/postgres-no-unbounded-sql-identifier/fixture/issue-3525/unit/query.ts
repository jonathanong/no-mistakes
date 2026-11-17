import sql from 'sql-template-strings';
export function missing(id: number) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  readMissing(statement);
  readMatched(statement, q.values);
  readOptions(statement, q.values, { mode: 'read' });
  const values = q.values;
  readValuesAlias(statement, values);
  readLaterSpread(statement, q.values, ...[]);
  readEarlySpread(statement, ...[q.values]);
  const second = sql`SELECT * FROM items WHERE id = ${id}`;
  readWrong(statement, second.values);
  const forward = statement;
  const again = forward;
  readMulti(again);
  const builderAlias = q;
  readBuilderAlias(statement, builderAlias.values);
  const empty = sql`SELECT * FROM items`;
  const noBinds = `${empty.text}`;
  readFree(noBinds);
  readDirect(`/* alias */ ${q.text}`);
  const literal = sql`SELECT * FROM items WHERE id = ${1}`;
  const constant = `${literal.text}`;
  readConstant(constant);
}
export function mutable(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  let statement = `${q.text}`;
  readMutable(statement);
  const a = b;
  const b = a;
  readCycle(a);
  const missing = absent;
  readAbsent(missing);
}
export function shadow(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  {
    const q = sql`SELECT * FROM items WHERE id = ${id}`;
    readShadow(statement, q.values);
  }
}
export function mutation(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  q.append(' WHERE other = 1');
  readMutation(statement);
}
export function escaped(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  mutate(q);
  readEscape(statement);
}
export function wrappedEscape(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  consume({ query: [q] });
  readWrappedEscape(statement);
}
export function memberReads(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  consume({ text: q.text, values: [q.values] });
  readMemberReads(statement);
}
export const exportedBuilder = sql`SELECT * FROM items WHERE id = ${1}`;
const exportedStatement = `${exportedBuilder.text}`;
readExported(exportedStatement);
export function returnedBuilder(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  readReturned(statement);
  return { query: q };
}
const aliasedExportBuilder = sql`SELECT * FROM items WHERE id = ${1}`;
export const publicQuery = aliasedExportBuilder;
const aliasExportStatement = `${aliasedExportBuilder.text}`;
readExportedAlias(aliasExportStatement);
const wrappedExportBuilder = sql`SELECT * FROM items WHERE id = ${1}`;
export const publicQueries = { queries: [wrappedExportBuilder] };
const wrappedExportStatement = `${wrappedExportBuilder.text}`;
readExportedObject(wrappedExportStatement);
export function aliasWrite(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const alias = q;
  const statement = `${q.text}`;
  alias.text = 'SELECT 1';
  readAliasWrite(statement);
}
export function aliasCall(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const alias = q;
  const statement = `${q.text}`;
  mutate(alias);
  readAliasCall(statement);
}
export function opaqueProjection() {
  const statement = `${makeQuery().text}`;
  readOpaqueProjection(statement);
}
export function wrongProjection(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.values}`;
  readWrongProjection(statement);
}
export function objectBuilder() {
  const q = { text: 'SELECT 1', values: [] };
  const statement = `${q.text}`;
  readObjectBuilder(statement);
}
export function nonmemberValues(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  readNonmemberValues(statement, []);
}
export async function awaitedEscape(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  readAwaitedEscape(statement);
  return await q;
}
export function conditionalEscape(id: string, choose: boolean) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `${q.text}`;
  readConditionalEscape(statement);
  return choose ? q : null;
}
