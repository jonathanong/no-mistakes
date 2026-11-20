import { widgetColumns } from "./widgetColumns";

export function extend(extra: unknown) {
  const widgetColumns = extra;
  consume(widgetColumns);
}

function consume(value: unknown) {
  return value;
}
