import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(first: string, second: string) { return readdirSync(first ?? second); }
