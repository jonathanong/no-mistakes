import type { SessionExecutor } from "@example/db";

export async function scopedReceiver(client: { query: SessionExecutor }, query: SessionExecutor) {
  await client.query("SELECT 1", { query });
}
