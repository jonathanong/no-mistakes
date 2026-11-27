import { vi } from "vitest";
import { enqueue } from "@example/jobs";
(vi.fn<typeof enqueue> as typeof vi.fn)();
