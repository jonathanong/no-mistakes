import { readdir } from "node:fs/promises";
import * as os from "node:os";
function inner(os) { return readdir(os.tmpdir()); }
export function scan() { return inner({ tmpdir: () => process.cwd() }); }
