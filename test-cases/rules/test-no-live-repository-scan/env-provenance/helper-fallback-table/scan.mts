import { readdir } from "node:fs/promises";
import * as path from "node:path";
const getDataDir = () => process.env.DATA_DIR ?? './tmp/data';
export function scan(table = "table") {
  return readdir(path.join(getDataDir(), table));
}
