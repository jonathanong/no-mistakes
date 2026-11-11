import { readdir, mkdtemp } from "node:fs/promises";
import { join } from "node:path";
export async function scan() {
  const parent = process.env.TMPDIR;
  const root = await mkdtemp(join(parent, "impact."));
  return readdir(root);
}
