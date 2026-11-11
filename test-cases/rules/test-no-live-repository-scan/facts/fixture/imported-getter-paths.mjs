import { readdirSync } from "node:fs";
import { join } from "node:path";
import { getLocalDir } from "./getter-definitions.mjs";
readdirSync(join(getLocalDir(), "table"));
const captured = getLocalDir();
readdirSync(captured);
readdirSync(getLocalDir("argument"));
export function pure() { return process.env.X ?? "./tmp/data"; }
export function impure() { process.env.X = "."; return process.env.X; }
export function absolute() { return join(process.env.X, "/unowned"); }
