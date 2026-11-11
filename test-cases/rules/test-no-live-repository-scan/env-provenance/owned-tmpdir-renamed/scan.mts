import { readdir } from "node:fs/promises";
import { tmpdir as tempRoot } from "node:os";
export function scan() { return readdir(tempRoot()); }
