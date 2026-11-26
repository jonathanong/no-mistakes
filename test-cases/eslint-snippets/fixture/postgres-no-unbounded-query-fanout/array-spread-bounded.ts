import { read } from "@example/db";
declare const KNOWN_IDS: number[];
declare function chunkArray<T>(items: T[], size: number): T[][];

export async function boundedSpreads(rows: number[]) {
  await Promise.all([0, 1].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  await Promise.all([...KNOWN_IDS].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  await Promise.all([0, ...[1, 2]].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  const fixed = [0, 1];
  await Promise.all([...fixed, ...fixed].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  await Promise.all(
    [...chunkArray(rows, 10)].map((batch) => read("SELECT id FROM orders WHERE id = ANY($1)", [batch])),
  );
  for (const batch of chunkArray(rows, 10)) {
    await Promise.all([...batch].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  }
}
