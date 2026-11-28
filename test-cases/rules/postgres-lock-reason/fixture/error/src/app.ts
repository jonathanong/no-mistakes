import { query } from "@example/db";
import { reserveJob } from "./helpers";

export async function run(id: number) {
  await reserveJob(id);
  // lock-reason: prevents: double refund
  await query("SELECT id FROM refunds WHERE id = $1 FOR UPDATE", [id]);
}
