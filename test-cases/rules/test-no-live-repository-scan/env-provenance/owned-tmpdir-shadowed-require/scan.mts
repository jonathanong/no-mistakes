import { readdir } from "node:fs/promises";
const require = () => ({ tmpdir: () => process.cwd() });
const os = require("node:os");
export function scan() { return readdir(os.tmpdir()); }
