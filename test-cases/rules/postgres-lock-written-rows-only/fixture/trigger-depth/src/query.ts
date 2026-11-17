import { beginTransaction } from "@example/db";

export async function atDepthBound(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM hop_32 WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO hop_0 (id) VALUES ($1)", [id]);
}

export async function beyondDepthBound(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM hop_33 WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO hop_0 (id) VALUES ($1)", [id]);
}
