import { readdir } from "node:fs/promises";
const os = require("node:os");
export function scan() { return readdir(os.tmpdir()); }
