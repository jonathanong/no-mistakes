import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: { outputRoot: string }[]) { args[0].outputRoot = process.cwd(); return scan(...args); }
function scan({ outputRoot }: { outputRoot: string }) { return readdirSync(outputRoot); }
