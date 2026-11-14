import { query } from "@example/db";
import { openTransaction } from "../transaction";

export async function run() {
  const forward = (sql: string) => query(sql);
  forward("SELECT confirmed_first");
  const tx = await openTransaction();
  await tx("SELECT relative_second");
}
