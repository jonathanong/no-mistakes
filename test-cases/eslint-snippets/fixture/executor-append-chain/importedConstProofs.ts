import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as fragments from "./fragments";
import { JOINS, SELECT_COLS as COLUMNS } from "./fragments";

async function importedChain(query: SessionExecutor, id: string) {
  await query(
    sql`SELECT `
      .append(COLUMNS)
      .append(sql` FROM widgets`)
      .append(JOINS)
      .append(sql` WHERE id = ${id}`),
  );
}

async function namespaceChain(query: SessionExecutor, id: string) {
  await query(
    sql`SELECT `.append(fragments.SELECT_COLS).append(sql` FROM widgets WHERE id = ${id}`),
  );
}

export function viaImported(query: SessionExecutor, id: string) {
  return importedChain(query, id);
}
export function viaNamespace(query: SessionExecutor, id: string) {
  return namespaceChain(query, id);
}
