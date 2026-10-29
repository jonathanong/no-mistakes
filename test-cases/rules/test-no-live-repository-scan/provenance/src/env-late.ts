import { readdirSync } from "node:fs";

const getDir = () => process.env.LATE_DIR ?? "./tmp/late";

export function listLate() {
  return readdirSync(getDir());
}
