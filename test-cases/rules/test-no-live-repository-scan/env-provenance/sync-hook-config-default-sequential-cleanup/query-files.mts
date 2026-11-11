import { readdir } from "node:fs/promises";
import * as path from "node:path";
export function getLocalDir() { return process.env.X ?? "./tmp/data"; }
export function query(table) { return readdir(path.join(getLocalDir(), table)); }
