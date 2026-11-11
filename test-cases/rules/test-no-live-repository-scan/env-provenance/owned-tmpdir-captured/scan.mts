import { readdir } from "node:fs/promises";
import * as os from "node:os";
const captured = os.tmpdir();
export function scan() { return readdir(captured); }
