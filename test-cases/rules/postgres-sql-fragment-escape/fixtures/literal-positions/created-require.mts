import { createRequire } from "node:module";

const load = createRequire(import.meta.url);

export function appendColumns(extra: unknown) {
  const parts = load("./columns.js");
  parts.append(extra);
}
