import { vi } from "vitest";

const mocking = vi;
const other = () => {};
const factory = () => ({ charge() {} });

let mock = mocking.mock;
mock = other;
mock("./internal", factory);

let kept = mocking.mock;
kept("./kept", factory);

let early = mocking.mock;
early("./early", factory);
early = other;
