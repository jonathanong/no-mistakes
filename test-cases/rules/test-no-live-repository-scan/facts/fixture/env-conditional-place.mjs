import { beforeEach, vi } from "vitest";
const enabled = false;
enabled ? vi.stubEnv("T", "owned") : vi.stubEnv("U", "owned");
function hidden(flag) {
  if (flag) process.env.W = "owned";
}
beforeEach(() => {
  if (enabled) vi.stubEnv("H", "owned");
});
