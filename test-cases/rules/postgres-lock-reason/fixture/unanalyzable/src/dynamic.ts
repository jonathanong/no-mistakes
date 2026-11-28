import { query } from "@example/db";

export async function runDynamic(sql: string) {
  // lock-reason: prevents: dynamic sql must stay unanalyzable; writer: worker
  return query(sql);
}
