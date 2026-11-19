import sql from 'sql-template-strings';
import { read } from './db.mjs';
import { dynamicFrom, fixedJoin, makeSelection, selectedFields } from './projection.mjs';
export function listColumn(id: string, options: { query?: typeof read }) {
  const column = sql`id`;
  const statement = sql`/* listColumn */ SELECT `;
  statement.append(column);
  statement.append(sql` FROM painted_widgets WHERE id = ${id}`);
  return read(statement, options);
}
export function listWithFixedFragments(id: string, after?: string) {
  const statement = sql`/* listWithFixedFragments */ SELECT `;
  statement.append(selectedFields);
  statement.append(sql` FROM painted_widgets w `).append(fixedJoin);
  if (after) statement.append(sql` WHERE w.id > ${after}`);
  statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
export function listWithOptions(id: string, options: { query?: typeof read } | undefined) {
  const extra = sql`, EXISTS (SELECT 1 FROM gadget_links links WHERE links.widget_id = w.id AND links.gadget_id = ${id})`;
  const statement = sql`/* listWithOptions */`
    .append(makeSelection(sql` SELECT `, extra))
    .append(sql` WHERE w.id = ${id} LIMIT 1`);
  return read(statement, options);
}
export function rawIdentifier(table: string) {
  return read(`SELECT id FROM ${table}`);
}
export function importedIdentifier(table: string) {
  const statement = sql`SELECT id `.append(dynamicFrom(table));
  return read(statement);
}
