import { readdir } from "node:fs/promises";
import { join } from "node:path";
export function scan() { return readdir(join(process.cwd(), process.env.TMPDIR)); }
