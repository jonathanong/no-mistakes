import { join } from "node:path";
import { tmpdir } from "node:os";

export const literal = "/tmp/example.json";
export const joined = join(tmpdir(), "example.json");
