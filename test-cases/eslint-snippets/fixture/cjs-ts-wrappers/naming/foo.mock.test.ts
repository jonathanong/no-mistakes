import { vi } from "vitest";
(vi.mock as typeof vi.mock)("./dep");
(vi as typeof vi).mock("./dep");
vi["mock" as const]("./dep");
