import * as columns from "./widgetColumns";

export function extend(
  columns: { widgetColumns: { append(extra: unknown): void } },
  extra: unknown,
) {
  columns.widgetColumns.append(extra);
}
