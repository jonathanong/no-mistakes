import { vi } from "vitest";

(vi.mock as typeof vi.mock).call(vi, "./internal", () => ({ charge() {} }));
(vi.mock as typeof vi.mock).apply(vi, ["./internal", () => ({ charge() {} })] as const);
