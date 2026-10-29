import { readdirSync } from "node:fs";

const getDir = () => process.env.COND_DIR ?? "./tmp/cond";

export function listCond() {
  return readdirSync(getDir());
}
