import { widgetItems } from "@app/order-list";

export function extend(extra: unknown) {
  widgetItems.append(extra);
}
