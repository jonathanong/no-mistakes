import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
const api = vi;
api.fn = unknown;
vi.fn(() => enqueueEmail("1"));
