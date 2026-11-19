import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { putEvent } from "./loop-helper";

type Row = { id: string; stamp: string };
type Values = { id: string; at: Date };

async function put(tx: SessionExecutor, values: { id: string; at: Date }) {
  await tx(sql`INSERT INTO events (id, at) VALUES (${values.id}, ${values.at})`);
}

async function putDestructured(tx: SessionExecutor, { id, at }: Values) {
  await tx(sql`INSERT INTO events (id, at) VALUES (${id}, ${at})`);
}

async function putNamed(tx: SessionExecutor, values: Values) {
  const { at } = values;
  await tx(sql`INSERT INTO events (id, at) VALUES (${values.id}, ${at})`);
}

async function putDefault(tx: SessionExecutor, id: string, at: Date = new Date(0)) {
  await tx(sql`INSERT INTO events (id, at) VALUES (${id}, ${at})`);
}

export async function dateProperty(tx: SessionExecutor, row: Row) {
  await put(tx, { id: row.id, at: new Date() });
  await putDestructured(tx, { id: row.id, at: new Date() });
  await putNamed(tx, { id: row.id, at: new Date() });
  await put(tx, { at: new Date(), id: row.id });
  await put(tx, { id: row.id, at: new Date(0) });
  await put(tx, { id: row.id, at: new Date(row.stamp) });
}

export async function dateImported(tx: SessionExecutor, row: Row) {
  await putEvent(tx, { id: row.id, at: new Date() });
}

export async function dateArgument(tx: SessionExecutor, row: Row) {
  await putDefault(tx, row.id, new Date(0));
  await putDefault(tx, row.id);
}
