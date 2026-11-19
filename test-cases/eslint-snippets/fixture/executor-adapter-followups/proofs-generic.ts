import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

async function putGeneric<T extends { id: string }>(tx: SessionExecutor, values: T) {
  await tx(sql`UPDATE widgets SET id = ${values.id}`);
}

export async function genericParameter<T extends { id: string }>(tx: SessionExecutor, values: T) {
  await putGeneric(tx, values);
}
