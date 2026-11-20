import * as orders from "./orders/items";
import * as widgets from "./widgets/items";

export function extend(extra: unknown, query: { append(value: unknown): unknown }) {
  orders.orderItems.append(extra);
  return query.append(widgets.widgetItems);
}
