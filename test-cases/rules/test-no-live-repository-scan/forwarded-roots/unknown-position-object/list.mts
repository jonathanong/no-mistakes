import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(first: unknown, { outputRoot }: { outputRoot: string }) { return readdirSync(outputRoot); }
