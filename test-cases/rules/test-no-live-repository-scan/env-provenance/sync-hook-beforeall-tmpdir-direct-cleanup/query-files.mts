import { readdirSync } from "node:fs";
import * as path from "node:path";
export function query(table) { return readdirSync(process.env.TMPDIR); }
