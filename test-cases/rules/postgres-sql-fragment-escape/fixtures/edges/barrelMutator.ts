import { widgetColumns } from "./barrel";

export function extend(extra: unknown) {
  widgetColumns.append(extra);
}
