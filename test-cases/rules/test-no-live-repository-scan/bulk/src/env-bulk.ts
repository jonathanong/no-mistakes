import { readdirSync } from "node:fs";

const getDir = () => process.env.BULK_DIR ?? "./tmp/bulk";

export function listBulk() {
  return readdirSync(getDir());
}
