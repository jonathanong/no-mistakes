import { readdir } from "node:fs/promises";
const { tmpdir } = require("os");
export function scan() { return readdir(tmpdir()); }
