import { query } from "@example/db";

declare function chunkArray<T>(items: T[], size: number): T[][];
declare const ids0: string[];
declare const o: { all(): string[] };

for (let ids of chunkArray(ids0, 5)) {
  ids = o.all();
  await Promise.all(ids.map((id) => query("SELECT 1 WHERE id=$1", [id])));
}
