import { readdirSync } from "node:fs";

const getDir = () => process.env.MIXED_DIR ?? "./tmp/mixed";

export function listMixed() {
  return readdirSync(getDir());
}
