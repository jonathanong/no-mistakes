import { vi } from "vitest";

const mocking = vi as typeof vi;
mocking.mock("./local", () => ({ charge() {} }));
mocking.mock.apply(mocking, ["./local", () => ({ charge() {} })] as const);
