import sql from "sql-template-strings";
import { read } from "@example/db";
import { branching, looping, throwing } from "./visibility-sql.mts";

// Same-file helpers are straight-line only; imported ones follow the same rule.
export async function viaBranch(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(branching());
  return read(query);
}

export async function viaLoop(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(looping());
  return read(query);
}

export async function viaEarlyThrow(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(throwing());
  return read(query);
}
