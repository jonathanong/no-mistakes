import { read } from "@example/db";

export async function inspect(rows: number[]) {
  await Promise.all(rows.map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  await Promise.all([...rows].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  await Promise.all([0, ...rows].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  await Promise.all([0, 1].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
}

export async function nestedAndAliasedSpreads(rows: number[]) {
  await Promise.all([0, ...[1, ...rows]].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  const copied = [...rows];
  await Promise.all(copied.map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  let reassigned = [0, 1];
  reassigned = rows;
  await Promise.all([...reassigned].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
  const loopA: number[] = [...loopB];
  const loopB: number[] = [...loopA];
  await Promise.all([...loopA].map((id) => read("SELECT id FROM orders WHERE id = $1", [id])));
}
