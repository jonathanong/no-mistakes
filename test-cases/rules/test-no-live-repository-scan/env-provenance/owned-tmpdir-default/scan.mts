import { readdir } from "node:fs/promises";
import os from "node:os";
export function scan() { return readdir(os.tmpdir()); }
