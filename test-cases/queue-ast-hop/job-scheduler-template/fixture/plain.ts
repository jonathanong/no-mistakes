declare function getOrders(): {
  upsertJobScheduler(...args: unknown[]): unknown;
};

export function schedulePlain() {
  const orders = getOrders();
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: { delay: 15 },
  });
}
