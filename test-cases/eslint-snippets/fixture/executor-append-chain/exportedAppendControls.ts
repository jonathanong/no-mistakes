import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const widgetColumns = sql`id, state`;

async function appended(query: SessionExecutor, id: string) {
  await query(
    sql`SELECT `
      .append(widgetColumns.append(sql`, extra`))
      .append(sql` FROM widgets WHERE id = ${id}`),
  );
}

export function viaAppended(query: SessionExecutor, id: string) {
  return appended(query, id);
}
