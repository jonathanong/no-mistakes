import { beginTransaction } from "@example/db";

export async function changeJobs() {
  const tx = await beginTransaction();
  await tx.query("LOCK TABLE jobs IN SHARE MODE");
  await tx.query("ALTER TABLE jobs ADD COLUMN title text");
}
