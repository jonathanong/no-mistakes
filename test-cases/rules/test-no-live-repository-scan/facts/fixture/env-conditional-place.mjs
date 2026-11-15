import { beforeEach, vi } from "vitest";
const enabled = false;
enabled ? vi.stubEnv("T", "owned") : vi.stubEnv("U", "owned");
function hidden(flag) {
  if (flag) process.env.W = "owned";
}
beforeEach(() => {
  if (enabled) vi.stubEnv("H", "owned");
});
function consume(value) {
  return value;
}
if (enabled) consume(vi.stubEnv("X", "owned"));
if (vi.stubEnv("Y", "owned")) {
}
consume(enabled ? vi.stubEnv("Z", "owned") : undefined);
