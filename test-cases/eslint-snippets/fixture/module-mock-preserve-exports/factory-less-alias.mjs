import { vi } from "vitest";

const mocking = vi;
const mock = mocking.mock;

mock.call(vi, "./internal");
mock.apply(vi, ["./internal"]);
mock.call(vi, "./kept", () => ({ charge() {} }));
