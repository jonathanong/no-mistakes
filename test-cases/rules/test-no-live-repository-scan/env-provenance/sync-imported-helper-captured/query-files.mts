import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { getLocalDir } from "./get-local-dir.mjs";
const captured = getLocalDir();
export function query(table) { return readdir(path.join(captured, table)); }
