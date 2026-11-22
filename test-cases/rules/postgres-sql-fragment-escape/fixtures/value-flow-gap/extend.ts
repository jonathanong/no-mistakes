import { widgetColumns } from "./columns";
import { widgetNote } from "./wide";

export function extend(extra: unknown) {
  widgetColumns.append(extra);
}
