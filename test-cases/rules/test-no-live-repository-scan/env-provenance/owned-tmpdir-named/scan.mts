import { readdir } from "node:fs/promises";
import { tmpdir } from "node:os";
export function scan() { return readdir(tmpdir()); }
