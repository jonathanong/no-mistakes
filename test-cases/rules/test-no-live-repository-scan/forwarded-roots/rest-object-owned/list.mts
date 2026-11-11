import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: { outputRoot: string }[]) { return scan(...args); }
function scan({ outputRoot }: { outputRoot: string }) { return readdirSync(outputRoot); }
