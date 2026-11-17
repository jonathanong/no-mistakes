import { query } from "@example/db";

export async function lockAccounts() {
  await query("LOCK TABLE accounts IN SHARE MODE");
}
