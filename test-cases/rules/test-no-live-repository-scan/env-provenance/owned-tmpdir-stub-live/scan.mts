import { readdir } from "node:fs/promises";
import * as os from "node:os";
export function scan() { return readdir(os.tmpdir()); }
