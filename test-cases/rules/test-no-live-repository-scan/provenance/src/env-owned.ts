import { readdirSync } from "node:fs";
import path from "node:path";

function getLocalDir() {
  return process.env.OWNED_DIR ?? "./tmp/default";
}

export function listTable(table: string) {
  return readdirSync(path.join(getLocalDir(), table));
}
