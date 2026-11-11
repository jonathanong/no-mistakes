import { readdir } from "node:fs/promises";
function tmpdir() { return process.cwd(); }
export function scan() { return readdir(tmpdir()); }
