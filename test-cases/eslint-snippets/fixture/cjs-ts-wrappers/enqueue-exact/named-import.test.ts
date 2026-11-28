import { test as V } from "vitest";
import { enqueue } from "@example/jobs";

V.vi.mocked(enqueue);
