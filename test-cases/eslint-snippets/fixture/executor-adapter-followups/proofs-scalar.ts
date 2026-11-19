import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; at: Date };

async function mark(tx: SessionExecutor, values: { id: string; at: Date }) {
  await tx(sql`INSERT INTO events (id, at) VALUES (${values.id}, ${values.at})`);
}

async function lockNotice(tx: SessionExecutor, id: string) {
  await tx(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

export async function nonRowDate(tx: SessionExecutor, id: string, at: Date) {
  const { rows } = await tx<{ rows: { id: string }[] }>(
    sql`SELECT id FROM events WHERE id = ${id}`,
  );
  const row = rows[0]!;
  await mark(tx, { id: row.id, at });
  await mark(tx, { id: row.id, at: new Date() });
}

export async function rowId(tx: SessionExecutor, rows: Notice[]) {
  await Promise.all(rows.map(({ id }) => lockNotice(tx, id)));
}

export async function dateParameter(tx: SessionExecutor, at: Date) {
  await mark(tx, { id: "a", at });
}
