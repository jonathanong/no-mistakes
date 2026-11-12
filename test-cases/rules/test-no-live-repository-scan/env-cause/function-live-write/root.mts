import { it, vi } from "vitest";
import { scan } from "./scan.mjs";

it("assigns the repository as the root", () => {
  vi.stubEnv("EXAMPLE_LOCAL_DIR", process.cwd());
  scan();
});
