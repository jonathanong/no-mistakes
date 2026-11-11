import { readdir } from "node:fs/promises";
const os = { tmpdir: () => process.cwd() };
export function scan() { return readdir(os.tmpdir()); }
