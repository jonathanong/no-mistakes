import { vi } from "vitest";
import { enqueueEmail } from "./queue";
import { enqueueAudit } from "./audit";

vi.mock("./queue");
vi.mock("./audit");

vi.mocked(enqueueEmail).mockResolvedValue(undefined);
vi.mocked(enqueueAudit).mockResolvedValue(undefined);
