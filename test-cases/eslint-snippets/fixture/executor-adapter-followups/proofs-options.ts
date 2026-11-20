import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type RunOptions = { query?: SessionExecutor; readOnly?: boolean };

async function bangInline(id: string, options: { query?: SessionExecutor }) {
  await options.query!(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

async function bangNamed(id: string, options: RunOptions) {
  await options.query!(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function forwardBang(query: SessionExecutor, id: string) {
  await bangInline(id, { query });
  await bangNamed(id, { query });
}
