import { readdirSync } from "node:fs";

const getDir = () => process.env.UNWRITTEN_DIR ?? "./tmp/unwritten";

export function listUnwritten() {
  return readdirSync(getDir());
}
