import { readdir, mkdtemp } from "node:fs/promises";
import * as os from "node:os";
import { dirname, join } from "node:path";
export async function scan() { return readdir(dirname(join(os.tmpdir(), "child"))); }
