import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as ns from "./elementControls";

async function keepLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(ns["ELEMENT_KEEP"]).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaElementKeepUse(query: SessionExecutor, id: string) {
  return keepLocal(query, id);
}
