import { query } from "@example/db";

declare const userIds: string[];

for (const ids of [userIds]) {
  await Promise.all(ids.map((id) => query("SELECT 1 WHERE id=$1", [id])));
}
