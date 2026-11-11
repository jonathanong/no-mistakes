import { readdir } from "node:fs/promises";
function getOS() { return { tmpdir: () => process.cwd() }; }
export function scan() { return readdir(getOS().tmpdir()); }
