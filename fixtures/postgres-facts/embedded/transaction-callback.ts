import { transaction, ordinary } from "@example/db";

export async function update() {
  await transaction(async (tx) => {
    await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  });
  await ordinary(async (tx) => {
    await tx.query("SELECT id FROM items WHERE id = 2 FOR UPDATE");
  });
  await transaction(async (outer) => {
    await transaction(async (inner) => {
      await inner.query("SELECT id FROM items WHERE id = 3 FOR UPDATE");
    });
    await outer.query("SELECT id FROM items WHERE id = 4 FOR UPDATE");
  });
  async function shadowed(transaction: (callback: (tx: { query: Function }) => Promise<void>) => Promise<void>) {
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 5 FOR UPDATE");
    });
  }
  {
    const transaction = ordinary;
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 6 FOR UPDATE");
    });
  }
  await transaction(async (tx) => {
    tx = ordinary;
    await tx.query("SELECT id FROM items WHERE id = 7 FOR UPDATE");
  });
}
