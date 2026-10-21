import { query } from "@example/db";

declare function chunkArray<T>(items: T[], size: number): T[][];
declare const userIds: string[];

for (const ids of chunkArray(userIds, 50)) {
  await Promise.all(ids.map((id) => query("SELECT * FROM users WHERE id = $1", [id])));
}
