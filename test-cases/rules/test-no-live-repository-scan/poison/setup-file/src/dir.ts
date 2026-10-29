import { readdirSync } from "node:fs";

const getDir = () => process.env.POISON_DIR ?? "./tmp/poison";

export function list() {
  return readdirSync(getDir());
}
