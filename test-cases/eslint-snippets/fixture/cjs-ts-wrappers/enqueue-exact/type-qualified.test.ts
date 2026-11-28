import type * as jobs from "@example/jobs";
import { vi } from "vitest";
vi.fn<typeof jobs.enqueue>();
