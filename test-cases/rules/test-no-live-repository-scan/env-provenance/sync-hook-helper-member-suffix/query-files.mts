import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { getLocalDir } from "./get-local-dir.mjs";
export function query(options) { return readdir(path.join(getLocalDir(), options.table)); }
