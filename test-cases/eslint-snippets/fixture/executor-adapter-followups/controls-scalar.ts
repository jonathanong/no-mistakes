import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; at: Date };
type Brand = string & { readonly brand: "Id" };

async function mark(tx: SessionExecutor, values: { id: string; at: Date }) {
  await tx(sql`INSERT INTO events (id, at) VALUES (${values.id}, ${values.at})`);
}

async function saveBrand(tx: SessionExecutor, values: { id: Brand }) {
  await tx(sql`UPDATE widgets SET id = ${values.id}`);
}

export async function rowDate(tx: SessionExecutor, id: string) {
  const { rows } = await tx<{ rows: Notice[] }>(sql`SELECT id, at FROM events WHERE id = ${id}`);
  await Promise.all(rows.map(({ at }) => mark(tx, { id, at })));
}

export async function branded(tx: SessionExecutor, id: Brand) {
  await saveBrand(tx, { id });
}
