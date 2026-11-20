import { widgetColumns } from "./widgetColumns";

export function extend(extra: unknown) {
  widgetColumns.text = extra;
}
