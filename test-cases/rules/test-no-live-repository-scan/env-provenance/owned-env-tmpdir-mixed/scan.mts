import { readdir } from "node:fs/promises";
export function scan() { return readdir(process.env.TMPDIR); }
