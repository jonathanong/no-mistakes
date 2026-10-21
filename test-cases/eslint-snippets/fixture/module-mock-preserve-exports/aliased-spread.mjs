import { vi } from "vitest";

const mocking = vi;
const mock = mocking.mock;
const factory = () => ({ charge() {} });

mock.apply(...[vi, ["react", factory]], ["./internal", factory]);
mock.apply(vi, ["./local", ...[factory]]);
mock.call(...[vi, "./internal", factory]);
mock.apply(vi, ["./internal", factory]);
mock.call(vi, "./kept", factory);
