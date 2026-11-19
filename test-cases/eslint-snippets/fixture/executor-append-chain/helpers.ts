import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { JOINS, SELECT_COLS } from "./fragments";

const LOCAL_COLS = sql`id, state`;

export async function exportedHelper(query: SessionExecutor, id: string) {
  await query(
    sql`SELECT `
      .append(SELECT_COLS)
      .append(sql` FROM widgets`)
      .append(JOINS)
      .append(sql` WHERE id = ${id}`),
  );
}

export async function localConstHelper(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(LOCAL_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}
