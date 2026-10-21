import { query } from "@example/db";

declare function chunkArray<T>(items: T[], size: number): T[][];
declare const ids0: string[];
declare const o: { get(): string[] };

for (const c of chunkArray(ids0, 5)) {
  var ids;
  ids = o.get();
  await Promise.all(ids.map((id) => query("SELECT 1 WHERE id=$1", [id])));
}
