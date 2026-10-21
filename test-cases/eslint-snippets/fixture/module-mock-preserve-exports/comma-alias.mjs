import { vi } from "vitest";

const mocking = vi;
const mock = mocking.mock;
const factory = () => ({ charge() {} });

(0, mock)("./internal", factory);
(0, mock).call(null, "./kept", factory);
(0, mock.call)(null, "./other", factory);
