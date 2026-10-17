import sql from "sql-template-strings";
import { read } from "@example/db";
import { viaDiamond, precedence, viaDefault, viaImportThenExport } from "./index.mts";

// Reached by two `export *` paths that end at the same declaration.
export async function diamond(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(viaDiamond("l.owner_id"));
  return read(query);
}

// An explicit re-export wins over a star that offers the same name.
export async function explicitWins(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(precedence("l.owner_id"));
  return read(query);
}

export async function renamedDefault(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(viaDefault("l.owner_id"));
  return read(query);
}

export async function importedThenExported(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(viaImportThenExport("l.owner_id"));
  return read(query);
}
