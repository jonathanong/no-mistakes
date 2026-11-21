import * as orders from "orders-widget";

export function extend(extra: unknown) {
  Object.assign(orders, { extra });
}
