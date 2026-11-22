import { createRequire } from "node:module";

const load = createRequire(import.meta.url);

export function appendColumns(extra: unknown) {
  const parts = load("./columns.js");
  parts.append(extra);
  const resolvedPath = load.resolve("./columns.js");
  resolvedPath.widgetColumns.append(extra);
}
