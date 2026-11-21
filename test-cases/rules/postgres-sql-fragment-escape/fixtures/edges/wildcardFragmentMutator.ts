import { widgetItems } from "@widgets/items";

export function extend(extra: unknown) {
  widgetItems.append(extra);
}
