import { readdir } from "node:fs/promises";
const loader = { require: () => ({ tmpdir: () => process.cwd() }) };
const os = loader.require("node:os");
export function scan() { return readdir(os.tmpdir()); }
