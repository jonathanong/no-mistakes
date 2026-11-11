import { readdir } from "node:fs/promises";
import { tmpdir } from "./custom.mjs";
export function scan() { return readdir(tmpdir()); }
