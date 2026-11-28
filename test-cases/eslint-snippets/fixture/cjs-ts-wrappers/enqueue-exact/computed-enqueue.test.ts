import { vi } from "vitest";

vi.mocked(require("@example/jobs")["enqueue"]);
