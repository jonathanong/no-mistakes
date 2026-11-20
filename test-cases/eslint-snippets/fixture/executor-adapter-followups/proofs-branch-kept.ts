import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

async function put(tx: SessionExecutor, values: { id: string; ownerId?: string }) {
  await tx(sql`UPDATE widgets SET owner_id = ${values.ownerId} WHERE id = ${values.id}`);
}

export async function keepBranch(
  flag: boolean,
  left: SessionExecutor,
  right: SessionExecutor,
  id: string,
) {
  const { rows } = await left<{ rows: { id: string }[] }>(
    sql`SELECT id FROM widgets WHERE id = ${id}`,
  );
  const row = rows[0]!;
  await put(flag ? left : right, { id: row.id, ownerId: id });
}
