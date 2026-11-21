import * as columns from "./widgetColumns";

declare function chooseKey(): string;

export function extend(extra: unknown) {
  columns[chooseKey()].append(extra);
}
