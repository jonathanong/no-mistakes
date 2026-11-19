import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

async function putIndex(tx: SessionExecutor, values: { [key: string]: string }) {
  await tx(sql`UPDATE widgets SET extra = ${values.extra} WHERE id = ${values.id}`);
}

export async function indexSignature(tx: SessionExecutor, values: { [key: string]: string }) {
  await putIndex(tx, values);
}
