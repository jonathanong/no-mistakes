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
  async function shadowedDestructured({ transaction }: { transaction: Function }) {
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 8 FOR UPDATE");
    });
  }
  {
    const { transaction } = { transaction: ordinary };
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 9 FOR UPDATE");
    });
  }
  {
    const { ordinary: ignored, ...transaction } = { ordinary: 0, transaction: ordinary };
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 10 FOR UPDATE");
    });
  }
  async function shadowedRest(...transaction: Function[]) {
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 11 FOR UPDATE");
    });
  }
  {
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 12 FOR UPDATE");
    });
    function transaction(callback: Function) {}
  }
  try {
    throw new Error("caught");
  } catch (transaction) {
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 13 FOR UPDATE");
    });
  }
  const namedFunction = function transaction() {
    return transaction(async (tx) => {
      return tx.query("SELECT id FROM items WHERE id = 14 FOR UPDATE");
    });
  };
  const namedClass = class transaction {
    method() {
      return transaction(async (tx) => {
        return tx.query("SELECT id FROM items WHERE id = 15 FOR UPDATE");
      });
    }
  };
  async function shadowedDefault(
    value = transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 16 FOR UPDATE");
    }),
    transaction: Function = ordinary,
  ) {}
  async function parameterEnvironment(
    pending = transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 17 FOR UPDATE");
    }),
  ) {
    var transaction = ordinary;
    await transaction(async (tx) => {
      await tx.query("SELECT id FROM items WHERE id = 18 FOR UPDATE");
    });
    await pending;
  }
}
