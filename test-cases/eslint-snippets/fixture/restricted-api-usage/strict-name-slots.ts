declare const orders: { value: number };

interface Widget {
  [item: string]: number;
}

namespace widgets.item {}

const seen = orders.item;
const bare = item;
const kept = 1;
export { kept as item };
export { item };
export { item as default };
export type { item as widget };
export { type item as order };
export { item as order } from "./orders";
