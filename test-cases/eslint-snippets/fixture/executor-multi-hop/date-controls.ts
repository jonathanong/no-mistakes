import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Row = { id: string; stamp: string; blob: object };

async function put(tx: SessionExecutor, values: { id: string; at: Date }) {
  await tx(sql`INSERT INTO events (id, at) VALUES (${values.id}, ${values.at})`);
}

export async function dateFromBuilder(tx: SessionExecutor, row: Row) {
  await put(tx, { id: row.id, at: new Date(`${row.stamp}`) });
}

export async function dateFromObject(tx: SessionExecutor, row: Row) {
  await put(tx, { id: row.id, at: new Date(row.blob as never) });
}

export async function dateFromRow(tx: SessionExecutor, row: Row) {
  await put(tx, { id: row.id, at: row as never });
}
