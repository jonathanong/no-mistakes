import { widgetColumns } from "./widgetColumns";

export function extend() {
  consume(widgetColumns);
}

function consume(value: unknown) {
  return value;
}
