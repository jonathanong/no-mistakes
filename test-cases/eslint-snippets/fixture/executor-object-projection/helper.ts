import type { SessionExecutor } from "@example/db";
import sql from "@example/sql";

export type Input = { id: string; ownerId: string; at: string };

export async function markImported(executor: SessionExecutor, input: Input) {
  await executor(sql`UPDATE widgets SET at = ${input.at} WHERE id = ${input.id}`);
}

export async function markPositional(id: string, executor: SessionExecutor) {
  await executor(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}
