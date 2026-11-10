import { readdir } from "node:fs/promises";
import { join } from "node:path";

const DEFAULT_DIR = join(import.meta.dirname, "..", "..", "..", "x");

export async function fn(user: string, dir = DEFAULT_DIR) {
  void user;
  return readdir(dir);
}
