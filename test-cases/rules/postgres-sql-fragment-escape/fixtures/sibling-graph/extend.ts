import { widgetColumns } from "./columns";

export function extend(extra: unknown) {
  widgetColumns.append(extra);
}
