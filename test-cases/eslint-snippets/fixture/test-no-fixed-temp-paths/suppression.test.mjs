import { join } from "node:path";
import { tmpdir } from "node:os";

// no-mistakes-disable-next-line test-no-fixed-temp-paths -- tests a legacy path
export const first = "/tmp/example.json";
export const second = join(tmpdir(), "example.json"); // no-mistakes-disable-line no-mistakes/test-no-fixed-temp-paths -- fixture spelling
// no-mistakes-disable-next-line different-rule -- applies to another rule
export const third = join(tmpdir(), "still-fixed.json");
