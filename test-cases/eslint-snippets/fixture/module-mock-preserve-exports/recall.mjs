import { vi } from "vitest";

vi.mock.call(undefined, "./local", () => ({ charge() {} }));
vi.mock.apply(undefined, ["./local", () => ({ charge() {} })]);
vi[`mock`].call(undefined, "./local", () => ({ charge() {} }));
