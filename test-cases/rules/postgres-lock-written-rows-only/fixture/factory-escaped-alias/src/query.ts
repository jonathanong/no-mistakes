import { beginTransaction } from "@example/db";
declare const sink: { accept(value: unknown): void };
export async function escaped(id: string) {
  await using transaction = await beginTransaction();
  const execute = transaction;
  sink.accept(execute);
  await execute(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await transaction.commit();
}
