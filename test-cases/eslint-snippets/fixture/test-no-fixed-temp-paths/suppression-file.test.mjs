// no-mistakes-disable-file no-mistakes/test-no-fixed-temp-paths -- tests legacy paths
import { join } from "node:path";
import { tmpdir } from "node:os";

export const literal = "/tmp/example.json";
export const joined = join(tmpdir(), "example.json");
