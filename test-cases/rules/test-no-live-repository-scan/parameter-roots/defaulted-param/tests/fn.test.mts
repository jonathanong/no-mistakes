import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fn } from "../src/fn.mts";

const user = "user";
const dir = await mkdtemp(join(tmpdir(), "p-"));
try {
  await fn(user, dir);
} finally {
  await rm(dir, { recursive: true });
}
