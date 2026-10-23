import { enqueueEmail } from "@app/jobs";
vi.fn = unknown;
vi.fn(() => enqueueEmail("1"));
