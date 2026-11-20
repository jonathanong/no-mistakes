import { widgetColumns } from "./widgetColumns";

export function extend() {
  widgetColumns.strings.push("id");
  widgetColumns.values.splice(0, 0, "id");
}
