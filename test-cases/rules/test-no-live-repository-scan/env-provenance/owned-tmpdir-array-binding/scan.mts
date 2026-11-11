import { readdir } from "node:fs/promises";
const [tmpdir] = [() => process.cwd()];
export function scan() { return readdir(tmpdir()); }
