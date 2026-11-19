import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export interface QueryOptions {
  query?: SessionExecutor;
  readOnly?: boolean;
}

async function assertDelivered(id: string, options: QueryOptions = {}) {
  await options.query!(sql`SELECT ${id}`);
}

export async function rejectUnresolvable(id: string, options: QueryOptions = {}): Promise<never> {
  await assertDelivered(id, options);
  throw new Error("not found");
}
