import { vi } from "vitest";
import * as runner from "vitest";
vi.doMock[`apply`](vi, ["./a"]);
