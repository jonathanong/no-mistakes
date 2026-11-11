import { readdirSync, mkdtempSync } from "node:fs";
export function list(a?: unknown, b?: unknown, c?: unknown, d?: unknown, e?: unknown, f?: unknown, g?: unknown, h?: unknown, root = mkdtempSync("/tmp/wide-default-")) {
  return readdirSync(root);
}
