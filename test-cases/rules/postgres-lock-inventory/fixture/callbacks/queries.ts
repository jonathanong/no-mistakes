import { ordinary, transaction } from "@example/db";

export async function reserve() {
  await transaction(async (tx) => {
    await tx.query("SELECT id FROM jobs WHERE id = 1 FOR UPDATE");
  });
  await ordinary(async (tx) => {
    await tx.query("SELECT id FROM jobs WHERE id = 2 FOR SHARE");
  });
}
