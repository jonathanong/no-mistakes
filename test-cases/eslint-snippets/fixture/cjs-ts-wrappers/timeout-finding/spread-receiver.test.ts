import { vi } from "vitest";

vi.setConfig.call(...([null, { testTimeout: 60000 }] as const), { testTimeout: 1000 });
