import { query } from "@example/db";

export async function claim() {
  // lock-reason: prevents: duplicate claim; writer: worker retry
  return query("SELECT id FROM jobs FOR UPDATE");
}
