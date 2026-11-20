import { widgetColumns } from "./callSource";

export function extend(extra: unknown) {
  widgetColumns.append(extra);
}
