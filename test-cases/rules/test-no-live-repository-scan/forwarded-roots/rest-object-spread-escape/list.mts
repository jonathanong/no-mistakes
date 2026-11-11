import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: { outputRoot: string }[]) { mutate(...args); return scan(...args); }
function mutate(options: { outputRoot: string }) { options.outputRoot = process.cwd(); }
function scan({ outputRoot }: { outputRoot: string }) { return readdirSync(outputRoot); }
