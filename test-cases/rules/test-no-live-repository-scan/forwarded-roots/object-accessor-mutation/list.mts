import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(options: { outputRoot: string, side: unknown }) { const side = options.side; return readdirSync(options.outputRoot); }
